use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateTicketRequest, Role, Ticket, TicketAssigned, TicketStatusChanged, UpdateTicketRequest,
};
use crate::AppState;

pub async fn create_ticket(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTicketRequest>,
) -> AppResult<Json<Ticket>> {
    if payload.subject.trim().is_empty()
        || payload.subject.len() > 200
        || payload
            .description
            .as_ref()
            .is_some_and(|v| v.len() > 20000)
    {
        return Err(AppError::BadRequest(
            "Subject required (max 200); description max 20000".into(),
        ));
    }
    // org_id and created_by come from the verified JWT (auth_user),
    // NEVER from the request body — this is what makes cross-org
    // ticket creation impossible even if a malicious client tries it.
    let mut tx = state.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(auth_user.org_id)
        .execute(&mut *tx)
        .await?;
    if std::env::var("BETA_MODE").as_deref() == Ok("true") {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM tickets WHERE organization_id=$1")
                .bind(auth_user.org_id)
                .fetch_one(&mut *tx)
                .await?;
        if count >= 100 || payload.description.as_ref().is_some_and(|v| v.len() > 5000) {
            return Err(AppError::BadRequest(
                "Free beta limit: 100 tickets per workspace and 5000 bytes per description".into(),
            ));
        }
    }
    let ticket = sqlx::query_as::<_, Ticket>(r#"INSERT INTO tickets (organization_id, created_by, subject, description)
           VALUES ($1, $2, $3, $4)
           RETURNING id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel"#)
    .bind(auth_user.org_id)
    .bind(auth_user.user_id)
    .bind(payload.subject)
    .bind(payload.description)
    .fetch_one(&mut *tx)
    .await?;
    crate::handlers::platform::audit(
        &mut tx,
        &auth_user,
        "ticket.created",
        ticket.id,
        serde_json::json!({}),
    )
    .await?;
    crate::handlers::platform::event(
        &mut tx,
        &auth_user,
        "TicketCreated",
        ticket.id,
        serde_json::json!({"subject":ticket.subject}),
    )
    .await?;
    crate::workflow_engine::dispatch(&mut tx, &auth_user, "ticket.created", ticket.id).await?;
    tx.commit().await?;
    let ticket =
        sqlx::query_as::<_, Ticket>("SELECT * FROM tickets WHERE id=$1 AND organization_id=$2")
            .bind(ticket.id)
            .bind(auth_user.org_id)
            .fetch_one(&state.db)
            .await?;
    Ok(Json(ticket))
}

pub async fn list_tickets(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Ticket>>> {
    // Layer 1 (org isolation) is always applied: organization_id = $1.
    // Layer 2 (role-based ownership) is conditional:
    //   - customer: can only see tickets they personally created
    //   - agent/admin: can see every ticket in their org
    let tickets = match auth_user.role {
        Role::Customer => {
            sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
                   FROM tickets
                   WHERE organization_id = $1 AND created_by = $2
                   ORDER BY created_at DESC"#)
    .bind(auth_user.org_id)
    .bind(auth_user.user_id)
            .fetch_all(&state.db)
            .await?
        }
        Role::Agent | Role::Admin => {
            sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
                   FROM tickets
                   WHERE organization_id = $1
                   ORDER BY created_at DESC"#)
    .bind(auth_user.org_id)
            .fetch_all(&state.db)
            .await?
        }
    };

    Ok(Json(tickets))
}

pub async fn get_ticket(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> AppResult<Json<Ticket>> {
    // Layer 1: org isolation always applies.
    let ticket = sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .fetch_optional(&state.db)
    .await?
    // A row that exists in another org, or doesn't exist at all, both
    // return the SAME 404 — never leak "it exists but isn't yours" vs
    // "it doesn't exist" to the client.
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    // Layer 2: a customer may only view their own ticket.
    if auth_user.role == Role::Customer && ticket.created_by != auth_user.user_id {
        return Err(AppError::NotFound("Ticket not found".to_string()));
    }

    Ok(Json(ticket))
}

pub async fn update_ticket(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
    Json(payload): Json<UpdateTicketRequest>,
) -> AppResult<Json<Ticket>> {
    // RBAC: only agents/admins may modify a ticket at all.
    if auth_user.role == Role::Customer {
        return Err(AppError::Forbidden(
            "Customers cannot modify tickets".to_string(),
        ));
    }

    // Everything below happens inside ONE database transaction.
    // This is the core of the outbox pattern: the ticket update and
    // the outbox event rows are committed together, atomically.
    let mut tx = state.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(auth_user.org_id)
        .execute(&mut *tx)
        .await?;

    // Fetch the CURRENT state first
    let existing = sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
           FROM tickets
           WHERE id = $1 AND organization_id = $2
           FOR UPDATE"#)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    if let Some(to) = payload.status {
        if !crate::policy::transition(existing.status, to) {
            return Err(AppError::BadRequest("Invalid status transition".into()));
        }
    }
    if let Some(user) = payload.assigned_to {
        let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND organization_id=$2 AND role IN ('admin','agent'))").bind(user).bind(auth_user.org_id).fetch_one(&mut *tx).await?;
        if !valid {
            return Err(AppError::NotFound("Assignee not found".into()));
        }
    }
    let updated = sqlx::query_as::<_, Ticket>(r#"UPDATE tickets
           SET status = COALESCE($1, status),
               assigned_to = CASE WHEN $5 THEN NULL ELSE COALESCE($2, assigned_to) END
           WHERE id = $3 AND organization_id = $4
           RETURNING id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel"#)
    .bind(payload.status)
    .bind(payload.assigned_to)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .bind(payload.unassign)
    .fetch_one(&mut *tx)
    .await?;

    // Only write a domain event if that field actually changed
    if updated.status != existing.status {
        let event = TicketStatusChanged {
            ticket_id: updated.id,
            organization_id: updated.organization_id,
            old_status: existing.status,
            new_status: updated.status,
            changed_by: auth_user.user_id,
            changed_at: chrono::Utc::now(),
        };
        let payload_json = serde_json::to_value(&event)?;

        sqlx::query(
            r#"INSERT INTO outbox_events (event_type, aggregate_id, organization_id, payload)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind("TicketStatusChanged")
        .bind(updated.id)
        .bind(updated.organization_id)
        .bind(payload_json)
        .execute(&mut *tx)
        .await?;
    }

    if updated.assigned_to != existing.assigned_to {
        let event = TicketAssigned {
            ticket_id: updated.id,
            organization_id: updated.organization_id,
            assigned_to: updated.assigned_to,
            assigned_by: auth_user.user_id,
            assigned_at: chrono::Utc::now(),
        };
        let payload_json = serde_json::to_value(&event)?;

        sqlx::query(
            r#"INSERT INTO outbox_events (event_type, aggregate_id, organization_id, payload)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind("TicketAssigned")
        .bind(updated.id)
        .bind(updated.organization_id)
        .bind(payload_json)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("UPDATE tickets SET updated_at=now(),resolved_at=CASE WHEN status IN ('solved','closed') THEN coalesce(resolved_at,now()) ELSE NULL END WHERE id=$1 AND organization_id=$2").bind(ticket_id).bind(auth_user.org_id).execute(&mut *tx).await?;
    crate::handlers::platform::audit(
        &mut tx,
        &auth_user,
        "ticket.updated",
        ticket_id,
        serde_json::json!({"status":updated.status,"assigned_to":updated.assigned_to}),
    )
    .await?;
    // Persist all changes atomically
    if updated.status == crate::models::TicketStatus::Solved && existing.status != updated.status {
        crate::workflow_engine::dispatch(&mut tx, &auth_user, "ticket.resolved", ticket_id).await?;
    }
    tx.commit().await?;

    Ok(Json(updated))
}

// ---- AI Copilot proxy ----

#[derive(Debug, Serialize)]
struct CopilotSummarizeRequest {
    subject: String,
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketSummaryResponse {
    pub ticket_id: Option<String>,
    pub summary: String,
    pub key_points: Vec<String>,
    pub detected_sentiment: Option<String>,
}

pub async fn get_ticket_summary(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> AppResult<Json<TicketSummaryResponse>> {
    // RBAC: only agents/admins need AI summaries — customers don't manage tickets
    if auth_user.role == Role::Customer {
        return Err(AppError::Forbidden(
            "Customers cannot access ticket summaries".to_string(),
        ));
    }

    // Step 1: fetch the ticket, org-scoped — same pattern as get_ticket
    let ticket = sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    // Layer 2: a customer would only ever see their own ticket anyway,
    // but this route is agent/admin-only so no ownership check needed here.

    // Step 2: call ai-copilot — Rust is the ONLY thing that ever talks to it directly
    let content = ticket.description.clone().unwrap_or_default();
    let points = content
        .lines()
        .filter(|v| !v.trim().is_empty())
        .take(4)
        .map(|v| v.chars().take(300).collect::<String>())
        .collect::<Vec<_>>();
    let fallback = TicketSummaryResponse {
        ticket_id: Some(ticket_id.to_string()),
        summary: format!("{}: {}", ticket.subject, points.join(" ")),
        key_points: points,
        detected_sentiment: None,
    };
    let summary = crate::ai_activity::analyze(
        &state,
        &auth_user,
        ticket_id,
        "summarize",
        &CopilotSummarizeRequest {
            subject: ticket.subject,
            content: ticket.description,
        },
        fallback,
    )
    .await?;
    Ok(Json(summary))
}

// ---- Categorize ----

#[derive(Debug, Serialize)]
struct CopilotCategorizeRequest {
    subject: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketCategoryResponse {
    pub ticket_id: Option<String>,
    pub category: String,
    pub confidence: f32,
    pub priority: String,
    pub tags: Vec<String>,
    pub reasoning: Option<String>,
}

pub async fn get_ticket_category(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> AppResult<Json<TicketCategoryResponse>> {
    if auth_user.role == Role::Customer {
        return Err(AppError::Forbidden(
            "Customers cannot access ticket categorization".to_string(),
        ));
    }

    let ticket = sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let text = format!(
        "{} {}",
        ticket.subject,
        ticket.description.as_deref().unwrap_or("")
    )
    .to_lowercase();
    let category = if ["invoice", "refund", "payment", "billing"]
        .iter()
        .any(|word| text.contains(word))
    {
        "Billing"
    } else if ["login", "password", "account"]
        .iter()
        .any(|word| text.contains(word))
    {
        "Account"
    } else if ["bug", "error", "outage"]
        .iter()
        .any(|word| text.contains(word))
    {
        "Technical"
    } else {
        "General"
    };
    let fallback=TicketCategoryResponse{ticket_id:Some(ticket_id.to_string()),category:category.into(),confidence:0.0,priority:"MEDIUM".into(),tags:vec![category.to_lowercase()],reasoning:Some("Deterministic local fallback; service unavailable. Uncalibrated keyword rule, human review required.".into())};
    let category = crate::ai_activity::analyze(
        &state,
        &auth_user,
        ticket_id,
        "categorize",
        &CopilotCategorizeRequest {
            subject: ticket.subject,
            content: ticket.description.unwrap_or_default(),
        },
        fallback,
    )
    .await?;
    Ok(Json(category))
}

// ---- Draft Reply ----

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketDraftReplyResponse {
    pub ticket_id: Option<String>,
    pub draft_reply: String,
    pub suggested_action: Option<String>,
    pub tone_used: String,
}

pub async fn get_ticket_draft_reply(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> AppResult<Json<TicketDraftReplyResponse>> {
    if auth_user.role == Role::Customer {
        return Err(AppError::Forbidden(
            "Customers cannot access draft replies".to_string(),
        ));
    }

    let ticket = sqlx::query_as::<_, Ticket>(r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status, created_at, updated_at, priority, labels, channel
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#)
    .bind(ticket_id)
    .bind(auth_user.org_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let _ = ticket;
    let Json(proposal) =
        crate::handlers::platform::grounded(auth_user, State(state), Path(ticket_id)).await?;
    Ok(Json(TicketDraftReplyResponse {
        ticket_id: Some(ticket_id.to_string()),
        draft_reply: proposal["draft"].as_str().unwrap_or("").to_string(),
        suggested_action: Some(
            "Extractive evidence draft. Check relevance and citations before sending.".into(),
        ),
        tone_used: "extractive / human review required".into(),
    }))
}
