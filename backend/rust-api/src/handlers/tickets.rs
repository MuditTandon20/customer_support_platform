use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateTicketRequest, Role, Ticket, TicketAssigned, TicketStatus, TicketStatusChanged,
    UpdateTicketRequest,
};
use crate::AppState;

pub async fn create_ticket(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTicketRequest>,
) -> AppResult<Json<Ticket>> {
    // org_id and created_by come from the verified JWT (auth_user),
    // NEVER from the request body — this is what makes cross-org
    // ticket creation impossible even if a malicious client tries it.
    let ticket = sqlx::query_as!(
        Ticket,
        r#"INSERT INTO tickets (organization_id, created_by, subject, description)
           VALUES ($1, $2, $3, $4)
           RETURNING id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at"#,
        auth_user.org_id,
        auth_user.user_id,
        payload.subject,
        payload.description
    )
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
            sqlx::query_as!(
                Ticket,
                r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
                   FROM tickets
                   WHERE organization_id = $1 AND created_by = $2
                   ORDER BY created_at DESC"#,
                auth_user.org_id,
                auth_user.user_id
            )
            .fetch_all(&state.db)
            .await?
        }
        Role::Agent | Role::Admin => {
            sqlx::query_as!(
                Ticket,
                r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
                   FROM tickets
                   WHERE organization_id = $1
                   ORDER BY created_at DESC"#,
                auth_user.org_id
            )
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
    let ticket = sqlx::query_as!(
        Ticket,
        r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#,
        ticket_id,
        auth_user.org_id
    )
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

    // Fetch the CURRENT state first
    let existing = sqlx::query_as!(
        Ticket,
        r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
           FROM tickets
           WHERE id = $1 AND organization_id = $2
           FOR UPDATE"#,
        ticket_id,
        auth_user.org_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let updated = sqlx::query_as!(
        Ticket,
        r#"UPDATE tickets
           SET status = COALESCE($1, status),
               assigned_to = COALESCE($2, assigned_to)
           WHERE id = $3 AND organization_id = $4
           RETURNING id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at"#,
        payload.status as Option<TicketStatus>,
        payload.assigned_to,
        ticket_id,
        auth_user.org_id
    )
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

        sqlx::query!(
            r#"INSERT INTO outbox_events (event_type, aggregate_id, organization_id, payload)
               VALUES ($1, $2, $3, $4)"#,
            "TicketStatusChanged",
            updated.id,
            updated.organization_id,
            payload_json
        )
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

        sqlx::query!(
            r#"INSERT INTO outbox_events (event_type, aggregate_id, organization_id, payload)
               VALUES ($1, $2, $3, $4)"#,
            "TicketAssigned",
            updated.id,
            updated.organization_id,
            payload_json
        )
        .execute(&mut *tx)
        .await?;
    }

    // Persist all changes atomically
    tx.commit().await?;

    Ok(Json(updated))
}

// ---- AI Copilot proxy ----

#[derive(Debug, Serialize)]
struct CopilotSummarizeRequest {
    subject: String,
    content: Option<String>,
}

#[derive(Debug,Serialize, Deserialize)]
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
    let ticket = sqlx::query_as!(
        Ticket,
        r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#,
        ticket_id,
        auth_user.org_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    // Layer 2: a customer would only ever see their own ticket anyway,
    // but this route is agent/admin-only so no ownership check needed here.

    // Step 2: call ai-copilot — Rust is the ONLY thing that ever talks to it directly
    let copilot_url = format!("{}/copilot/summarize", state.ai_copilot_url);

    let response = state
        .http_client
        .post(&copilot_url)
        .json(&CopilotSummarizeRequest {
            subject: ticket.subject.clone(),
            content: ticket.description.clone(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::Internal("AI copilot returned an error".to_string()));
    }

    let summary: TicketSummaryResponse = response.json().await?;

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

    let ticket = sqlx::query_as!(
        Ticket,
        r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#,
        ticket_id,
        auth_user.org_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let copilot_url = format!("{}/copilot/categorize", state.ai_copilot_url);

    let response = state
        .http_client
        .post(&copilot_url)
        .json(&CopilotCategorizeRequest {
            subject: ticket.subject.clone(),
            content: ticket.description.clone().unwrap_or_default(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::Internal("AI copilot returned an error".to_string()));
    }

    let category: TicketCategoryResponse = response.json().await?;

    Ok(Json(category))
}

// ---- Draft Reply ----

#[derive(Debug, Serialize)]
struct CopilotDraftReplyRequest {
    subject: String,
    content: String,
}

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

    let ticket = sqlx::query_as!(
        Ticket,
        r#"SELECT id, organization_id, created_by, assigned_to, subject, description, status as "status: TicketStatus", created_at
           FROM tickets
           WHERE id = $1 AND organization_id = $2"#,
        ticket_id,
        auth_user.org_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Ticket not found".to_string()))?;

    let copilot_url = format!("{}/copilot/draft-reply", state.ai_copilot_url);

    let response = state
        .http_client
        .post(&copilot_url)
        .json(&CopilotDraftReplyRequest {
            subject: ticket.subject.clone(),
            content: ticket.description.clone().unwrap_or_default(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::Internal("AI copilot returned an error".to_string()));
    }

    let draft: TicketDraftReplyResponse = response.json().await?;

    Ok(Json(draft))
}