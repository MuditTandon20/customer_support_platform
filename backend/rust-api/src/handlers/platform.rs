//! Tenant authority for the connected support workspace. Never accept org IDs from request bodies.
use crate::{
    auth::{self, AuthUser},
    error::{AppError, AppResult},
    models::{Role, TicketStatus},
    policy::{self, Definition},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use sqlx::{PgConnection, Row};
use uuid::Uuid;

pub fn staff(a: &AuthUser) -> AppResult<()> {
    if a.role == Role::Customer {
        Err(AppError::Forbidden("Team access required".into()))
    } else {
        Ok(())
    }
}
pub async fn ai_activity(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(l) FROM ai_activity_log l WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 100").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn notifications(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    crate::outbox::project_for_org(&s.db, Some(a.org_id)).await?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(n)||jsonb_build_object('read',EXISTS(SELECT 1 FROM notification_reads r WHERE r.organization_id=n.organization_id AND r.notification_id=n.id AND r.user_id=$2)) FROM notifications n WHERE n.organization_id=$1 ORDER BY n.created_at DESC LIMIT 50").bind(a.org_id).bind(a.user_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn read_notification(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM notifications WHERE id=$1 AND organization_id=$2)",
    )
    .bind(id)
    .bind(a.org_id)
    .fetch_one(&s.db)
    .await?;
    if !exists {
        return Err(AppError::NotFound("Notification not found".into()));
    }
    sqlx::query("INSERT INTO notification_reads(organization_id,notification_id,user_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(a.org_id).bind(id).bind(a.user_id).execute(&s.db).await?;
    Ok(Json(json!({"read":true})))
}
fn admin(a: &AuthUser) -> AppResult<()> {
    if a.role != Role::Admin {
        Err(AppError::Forbidden("Administrator required".into()))
    } else {
        Ok(())
    }
}
fn text(s: &str, max: usize) -> AppResult<String> {
    let s = s.trim();
    if s.is_empty() || s.len() > max {
        Err(AppError::BadRequest(format!(
            "Text must contain 1–{max} bytes"
        )))
    } else {
        Ok(s.into())
    }
}
pub async fn audit(
    c: &mut PgConnection,
    a: &AuthUser,
    action: &str,
    id: Uuid,
    detail: Value,
) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_events(organization_id,actor_id,action,resource_id,detail) VALUES($1,$2,$3,$4,$5)")
        .bind(a.org_id).bind(a.user_id).bind(action).bind(id).bind(detail).execute(c).await?;
    Ok(())
}
pub async fn event(
    c: &mut PgConnection,
    a: &AuthUser,
    action: &str,
    id: Uuid,
    detail: Value,
) -> AppResult<()> {
    let payload = if action == "TicketStatusChanged" || action == "TicketAssigned" {
        detail
    } else {
        json!({"version":1,"organization_id":a.org_id,"resource_id":id,"data":detail})
    };
    sqlx::query("INSERT INTO outbox_events(organization_id,event_type,aggregate_id,payload) VALUES($1,$2,$3,$4)")
        .bind(a.org_id).bind(action).bind(id).bind(payload).execute(c).await?;
    Ok(())
}
async fn ticket(c: &mut PgConnection, a: &AuthUser, id: Uuid) -> AppResult<Value> {
    sqlx::query_scalar("SELECT to_jsonb(t) FROM tickets t WHERE id=$1 AND organization_id=$2 AND ($3 OR created_by=$4) FOR UPDATE")
        .bind(id).bind(a.org_id).bind(a.role!=Role::Customer).bind(a.user_id).fetch_optional(c).await?
        .ok_or_else(||AppError::NotFound("Ticket not found".into()))
}

#[derive(Deserialize)]
pub struct Search {
    #[serde(default)]
    pub q: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
}
pub async fn inbox(
    a: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<Search>,
) -> AppResult<Json<Value>> {
    let status = q.status.as_deref().unwrap_or("all");
    if ![
        "all",
        "active",
        "unassigned",
        "new",
        "open",
        "pending",
        "on_hold",
        "solved",
        "closed",
    ]
    .contains(&status)
    {
        return Err(AppError::BadRequest("Invalid inbox status".into()));
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(t) || jsonb_build_object('customer_name',u.full_name,'customer_email',u.email,'message_count',(SELECT count(*) FROM messages m WHERE m.organization_id=t.organization_id AND m.ticket_id=t.id AND (NOT m.internal OR $3))) FROM tickets t JOIN users u ON u.id=t.created_by AND u.organization_id=t.organization_id WHERE t.organization_id=$1 AND ($3 OR t.created_by=$4) AND ($7='all' OR ($7='active' AND t.status NOT IN ('solved','closed')) OR ($7='unassigned' AND t.assigned_to IS NULL) OR t.status::text=$7) AND (t.subject ILIKE $2 OR u.full_name ILIKE $2 OR u.email ILIKE $2 OR array_to_string(t.labels,' ') ILIKE $2 OR coalesce(t.description,'') ILIKE $2 OR EXISTS(SELECT 1 FROM messages m WHERE m.organization_id=t.organization_id AND m.ticket_id=t.id AND (NOT m.internal OR $3) AND m.body ILIKE $2)) ORDER BY t.updated_at DESC,t.id LIMIT $5 OFFSET $6")
        .bind(a.org_id).bind(format!("%{}%",q.q.chars().take(200).collect::<String>())).bind(a.role!=Role::Customer).bind(a.user_id).bind(q.limit.unwrap_or(50).clamp(1,100)).bind(q.offset.unwrap_or(0).clamp(0,10000)).bind(status).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn messages(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    ticket(&mut *s.db.acquire().await?, &a, id).await?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(m)-'idempotency_key' || jsonb_build_object('author_name',u.full_name,'author_role',u.role) FROM messages m JOIN users u ON u.id=m.author_id AND u.organization_id=m.organization_id WHERE m.organization_id=$1 AND m.ticket_id=$2 AND (NOT m.internal OR $3) ORDER BY m.created_at LIMIT 500")
        .bind(a.org_id).bind(id).bind(a.role!=Role::Customer).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub body: String,
    #[serde(default)]
    pub internal: bool,
    pub idempotency_key: Uuid,
    pub proposal_id: Option<Uuid>,
}
pub async fn reply(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Reply>,
) -> AppResult<Json<Value>> {
    let beta = std::env::var("BETA_MODE").as_deref() == Ok("true");
    let body = text(&p.body, if beta { 5000 } else { 20000 })?;
    if p.internal {
        staff(&a)?;
    }
    let mut tx = s.db.begin().await?;
    // Serialize same-tenant keys as well as concurrent posts to a thread.
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let t = ticket(&mut tx, &a, id).await?;
    if let Some(existing) = sqlx::query_scalar::<_, Value>(
        "SELECT to_jsonb(m) FROM messages m WHERE organization_id=$1 AND idempotency_key=$2",
    )
    .bind(a.org_id)
    .bind(p.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if existing["ticket_id"] != json!(id)
            || existing["body"] != body
            || existing["internal"] != p.internal
            || existing["author_id"] != json!(a.user_id)
            || existing["proposal_id"] != json!(p.proposal_id)
        {
            return Err(AppError::BadRequest(
                "Idempotency key used for a different message".into(),
            ));
        }
        return Ok(Json(existing));
    }
    if beta {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM messages WHERE organization_id=$1")
                .bind(a.org_id)
                .fetch_one(&mut *tx)
                .await?;
        if count >= 1000 {
            return Err(AppError::BadRequest(
                "The free beta message quota has been reached".into(),
            ));
        }
    }
    if t["status"] == "closed" {
        return Err(AppError::BadRequest(
            "Closed conversation cannot receive messages".into(),
        ));
    }
    if let Some(proposal_id) = p.proposal_id {
        staff(&a)?;
        let proposal:Value=sqlx::query_scalar("SELECT to_jsonb(p) FROM ai_proposals p WHERE organization_id=$1 AND ticket_id=$2 AND id=$3").bind(a.org_id).bind(id).bind(proposal_id).fetch_optional(&mut *tx).await?.ok_or_else(||AppError::NotFound("Proposal not found".into()))?;
        if proposal["abstained"] == true || proposal["draft"] != body {
            return Err(AppError::BadRequest(
                "Proposal is abstained or draft text has changed".into(),
            ));
        }
        for citation in proposal["citations"].as_array().into_iter().flatten() {
            let source: Uuid = citation["id"]
                .as_str()
                .unwrap_or("")
                .parse()
                .map_err(|_| AppError::BadRequest("Invalid citation".into()))?;
            let version:Option<i32>=sqlx::query_scalar("SELECT version FROM knowledge_documents WHERE organization_id=$1 AND id=$2 AND status='ready' AND (expires_at IS NULL OR expires_at>now()) FOR SHARE").bind(a.org_id).bind(source).fetch_optional(&mut *tx).await?;
            if version.map(|v| json!(v)) != Some(citation["version"].clone()) {
                return Err(AppError::BadRequest(
                    "Evidence changed or expired. Generate a fresh proposal before sending".into(),
                ));
            }
        }
    }
    let m:Value=sqlx::query_scalar("INSERT INTO messages(organization_id,ticket_id,author_id,body,internal,delivery,idempotency_key,proposal_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8) RETURNING to_jsonb(messages)")
        .bind(a.org_id).bind(id).bind(a.user_id).bind(&body).bind(p.internal).bind(if p.internal {"internal"}else{"portal_visible"}).bind(p.idempotency_key).bind(p.proposal_id).fetch_one(&mut *tx).await?;
    sqlx::query("UPDATE tickets SET updated_at=now() WHERE organization_id=$1 AND id=$2")
        .bind(a.org_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let detail = json!({"message_id":m["id"],"internal":p.internal,"delivery":m["delivery"]});
    audit(
        &mut tx,
        &a,
        if p.internal {
            "note.created"
        } else {
            "message.sent"
        },
        id,
        detail.clone(),
    )
    .await?;
    event(&mut tx, &a, "MessageCreated", id, detail).await?;
    if !p.internal && a.role != Role::Customer && t["channel"] == "email_sandbox" {
        let recipient: String =
            sqlx::query_scalar("SELECT email FROM users WHERE id=$1 AND organization_id=$2")
                .bind(t["created_by"].as_str().unwrap().parse::<Uuid>().unwrap())
                .bind(a.org_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query("INSERT INTO sandbox_mail(organization_id,ticket_id,recipient,subject,body,direction,idempotency_key) VALUES($1,$2,$3,$4,$5,'outbound',$6)").bind(a.org_id).bind(id).bind(recipient).bind(t["subject"].as_str().unwrap_or("")).bind(&body).bind(p.idempotency_key).execute(&mut *tx).await?;
    }
    if a.role == Role::Customer && !p.internal {
        crate::workflow_engine::dispatch(&mut tx, &a, "customer.replied", id).await?;
    }
    tx.commit().await?;
    Ok(Json(m))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub priority: String,
    pub labels: Vec<String>,
}
pub async fn metadata(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Metadata>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    if !["low", "normal", "high", "urgent"].contains(&p.priority.as_str())
        || p.labels.len() > 12
        || p.labels.iter().any(|v| v.trim().is_empty() || v.len() > 40)
    {
        return Err(AppError::BadRequest("Invalid priority or labels".into()));
    }
    let mut tx = s.db.begin().await?;
    ticket(&mut tx, &a, id).await?;
    sqlx::query("UPDATE tickets SET priority=$1,labels=$2,updated_at=now() WHERE id=$3 AND organization_id=$4")
        .bind(p.priority).bind(p.labels).bind(id).bind(a.org_id).execute(&mut *tx).await?;
    audit(&mut tx, &a, "ticket.metadata", id, json!({})).await?;
    tx.commit().await?;
    Ok(Json(json!({"updated":true})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Knowledge {
    pub title: String,
    pub content: String,
    pub visibility: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub async fn chunks(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM knowledge_documents WHERE organization_id=$1 AND id=$2 AND status='ready' AND (expires_at IS NULL OR expires_at>now()) AND ($3 OR visibility='public'))").bind(a.org_id).bind(id).bind(a.role!=Role::Customer).fetch_one(&s.db).await?;
    if !allowed {
        return Err(AppError::NotFound("Document not found".into()));
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'ordinal',ordinal,'content',content) FROM knowledge_chunks WHERE organization_id=$1 AND document_id=$2 ORDER BY ordinal").bind(a.org_id).bind(id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn knowledge(
    a: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<Search>,
) -> AppResult<Json<Value>> {
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(k)-'search_vector' FROM knowledge_documents k WHERE organization_id=$1 AND status='ready' AND (expires_at IS NULL OR expires_at>now()) AND ($2 OR visibility='public') AND (title ILIKE $3 OR content ILIKE $3) ORDER BY updated_at DESC LIMIT 100")
        .bind(a.org_id).bind(a.role!=Role::Customer).bind(format!("%{}%",q.q.chars().take(200).collect::<String>())).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn add_knowledge(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Knowledge>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let title = text(&p.title, 200)?;
    let beta = std::env::var("BETA_MODE").as_deref() == Ok("true");
    let content = text(&p.content, if beta { 20000 } else { 100000 })?;
    if !["team", "public"].contains(&p.visibility.as_str()) {
        return Err(AppError::BadRequest(
            "Visibility must be team or public".into(),
        ));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM knowledge_documents WHERE organization_id=$1 AND status='ready'",
    )
    .bind(a.org_id)
    .fetch_one(&mut *tx)
    .await?;
    if count >= if beta { 20 } else { 100 } {
        return Err(AppError::BadRequest(
            "Knowledge document limit reached for this testing workspace".into(),
        ));
    }
    let row:Value=sqlx::query_scalar("INSERT INTO knowledge_documents(organization_id,title,content,visibility,expires_at) VALUES($1,$2,$3,$4,$5) RETURNING to_jsonb(knowledge_documents)-'search_vector'")
        .bind(a.org_id).bind(title).bind(content).bind(p.visibility).bind(p.expires_at).fetch_one(&mut *tx).await?;
    audit(
        &mut tx,
        &a,
        "knowledge.created",
        row["id"].as_str().unwrap().parse().unwrap(),
        json!({"index":"postgres_full_text"}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(row))
}
pub async fn delete_knowledge(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let mut tx = s.db.begin().await?;
    let n=sqlx::query("UPDATE knowledge_documents SET status='archived',updated_at=now(),version=version+1 WHERE id=$1 AND organization_id=$2 AND status='ready'").bind(id).bind(a.org_id).execute(&mut *tx).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("Document not found".into()));
    }
    audit(&mut tx, &a, "knowledge.archived", id, json!({})).await?;
    tx.commit().await?;
    Ok(Json(json!({"archived":true})))
}
pub async fn reindex_knowledge(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let row:Value=sqlx::query_scalar("UPDATE knowledge_documents SET content=content,version=version+1,updated_at=now() WHERE id=$1 AND organization_id=$2 AND status='ready' RETURNING to_jsonb(knowledge_documents)-'search_vector'")
        .bind(id).bind(a.org_id).fetch_optional(&s.db).await?.ok_or_else(||AppError::NotFound("Document not found".into()))?;
    Ok(Json(row))
}
pub async fn grounded(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let started = std::time::Instant::now();
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let t = ticket(&mut tx, &a, id).await?;

    let count:i64=sqlx::query_scalar("SELECT count(*) FROM ai_proposals WHERE organization_id=$1 AND created_at>date_trunc('day',now())").bind(a.org_id).fetch_one(&mut *tx).await?;
    if count >= 200 {
        return Err(AppError::BadRequest(
            "Daily local draft quota reached (200)".into(),
        ));
    }
    let query = format!(
        "{} {}",
        t["subject"].as_str().unwrap_or(""),
        t["description"].as_str().unwrap_or("")
    );
    let query = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|v| v.len() > 2)
        .take(24)
        .collect::<Vec<_>>()
        .join(" OR ");
    let evidence:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',d.id,'chunk_id',c.id,'chunk_index',c.ordinal,'title',d.title,'excerpt',c.content,'version',d.version,'updated_at',d.updated_at) FROM knowledge_chunks c JOIN knowledge_documents d ON d.id=c.document_id AND d.organization_id=c.organization_id WHERE c.organization_id=$1 AND d.status='ready' AND (d.expires_at IS NULL OR d.expires_at>now()) AND (c.search_vector || to_tsvector('english',d.title)) @@ websearch_to_tsquery('english',$2) ORDER BY ts_rank(c.search_vector || to_tsvector('english',d.title),websearch_to_tsquery('english',$2)) DESC,c.ordinal LIMIT 3")
        .bind(a.org_id).bind(query).fetch_all(&mut *tx).await?;
    let abstained = evidence.is_empty();
    let draft = if abstained {
        "I do not have enough approved, current evidence to answer this question. Please review the request and add a relevant knowledge article.".into()
    } else {
        evidence
            .iter()
            .enumerate()
            .map(|(i, e)| {
                format!(
                    "[{}] {}\n{}",
                    i + 1,
                    e["title"].as_str().unwrap_or(""),
                    e["excerpt"].as_str().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    let result:Value=sqlx::query_scalar("INSERT INTO ai_proposals(organization_id,ticket_id,draft,citations,abstained) VALUES($1,$2,$3,$4,$5) RETURNING to_jsonb(ai_proposals)")
        .bind(a.org_id).bind(id).bind(draft).bind(json!(evidence)).bind(abstained).fetch_one(&mut *tx).await?;
    audit(
        &mut tx,
        &a,
        "copilot.proposed",
        id,
        json!({"abstained":abstained,"mode":"extractive","sources":evidence.len()}),
    )
    .await?;
    if abstained {
        crate::workflow_engine::dispatch(&mut tx, &a, "ai.abstained", id).await?;
    }
    crate::ai_activity::record(
        &mut tx,
        &a,
        id,
        "grounded_draft",
        "deterministic_extract",
        if abstained { "abstained" } else { "proposed" },
        started.elapsed().as_millis() as i64,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub ticket_id: Uuid,
    pub action: String,
    pub arguments: Value,
    pub idempotency_key: Uuid,
}
fn tool_valid(p: &Tool) -> AppResult<bool> {
    let obj = p
        .arguments
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Arguments must be an object".into()))?;
    let allowed = match p.action.as_str() {
        "ticket.label" => {
            obj.len() == 1
                && obj
                    .get("label")
                    .and_then(Value::as_str)
                    .is_some_and(|v| !v.trim().is_empty() && v.len() <= 40)
        }
        "ticket.status" => {
            obj.len() == 1
                && obj
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|v| serde_json::from_value::<TicketStatus>(json!(v)).is_ok())
        }
        "account.lookup" => obj.is_empty(),
        "refund" | "subscription.cancel" | "data.export" => {
            obj.len() == 1
                && obj
                    .get("reason")
                    .and_then(Value::as_str)
                    .is_some_and(|v| !v.trim().is_empty() && v.len() <= 1000)
        }
        _ => false,
    };
    if !allowed {
        return Err(AppError::BadRequest(
            "Unknown tool or invalid typed arguments".into(),
        ));
    }
    Ok(matches!(
        p.action.as_str(),
        "refund" | "subscription.cancel" | "data.export"
    ))
}
pub async fn tool_preview(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Tool>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let sensitive = tool_valid(&p)?;
    ticket(&mut *s.db.acquire().await?, &a, p.ticket_id).await?;
    Ok(Json(
        json!({"action":p.action,"arguments":p.arguments,"requires_admin_approval":sensitive,"external_adapter_available":!sensitive,"dry_run":true,"side_effects":false}),
    ))
}
pub async fn request_tool(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Tool>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let sensitive = tool_valid(&p)?;
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    ticket(&mut tx, &a, p.ticket_id).await?;

    if let Some(row) = sqlx::query_scalar::<_, Value>(
        "SELECT to_jsonb(e) FROM tool_executions e WHERE organization_id=$1 AND idempotency_key=$2",
    )
    .bind(a.org_id)
    .bind(p.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if row["ticket_id"] != json!(p.ticket_id)
            || row["action"] != p.action
            || row["arguments"] != p.arguments
        {
            return Err(AppError::BadRequest("Idempotency key conflict".into()));
        }
        return Ok(Json(row));
    }
    let id:Uuid=sqlx::query_scalar("INSERT INTO tool_executions(organization_id,ticket_id,requested_by,action,arguments,state,idempotency_key) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id")
        .bind(a.org_id).bind(p.ticket_id).bind(a.user_id).bind(&p.action).bind(&p.arguments).bind(if sensitive {"pending"}else{"executed"}).bind(p.idempotency_key).fetch_one(&mut *tx).await?;
    let result = if sensitive {
        json!({"reason":"Explicit admin approval required; external provider unavailable"})
    } else {
        execute_local(&mut tx, &a, p.ticket_id, &p.action, &p.arguments).await?
    };
    sqlx::query("UPDATE tool_executions SET result=$1 WHERE organization_id=$2 AND id=$3")
        .bind(&result)
        .bind(a.org_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &a,
        "tool.requested",
        id,
        json!({"action":p.action,"approval_required":sensitive}),
    )
    .await?;
    let row: Value = sqlx::query_scalar(
        "SELECT to_jsonb(e) FROM tool_executions e WHERE organization_id=$1 AND id=$2",
    )
    .bind(a.org_id)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(row))
}
pub(crate) async fn execute_local(
    c: &mut PgConnection,
    a: &AuthUser,
    id: Uuid,
    action: &str,
    args: &Value,
) -> AppResult<Value> {
    match action {
        "ticket.label" => {
            let changed=sqlx::query("UPDATE tickets SET labels=CASE WHEN $1=ANY(labels) THEN labels ELSE array_append(labels,$1) END,updated_at=now() WHERE organization_id=$2 AND id=$3 AND (array_length(labels,1) IS NULL OR array_length(labels,1)<12 OR $1=ANY(labels))")
                .bind(args["label"].as_str().unwrap()).bind(a.org_id).bind(id).execute(&mut *c).await?;
            if changed.rows_affected() == 0 {
                return Err(AppError::BadRequest("Ticket label quota reached".into()));
            }
        }
        "ticket.status" => {
            let from: TicketStatus = sqlx::query_scalar(
                "SELECT status FROM tickets WHERE organization_id=$1 AND id=$2 FOR UPDATE",
            )
            .bind(a.org_id)
            .bind(id)
            .fetch_one(&mut *c)
            .await?;
            let to: TicketStatus = serde_json::from_value(args["status"].clone())?;
            if !policy::transition(from, to) {
                return Err(AppError::BadRequest(
                    "Invalid ticket status transition".into(),
                ));
            }
            sqlx::query("UPDATE tickets SET status=$1,updated_at=now(),resolved_at=CASE WHEN $1 IN ('solved','closed') THEN coalesce(resolved_at,now()) ELSE NULL END WHERE organization_id=$2 AND id=$3")
                .bind(to).bind(a.org_id).bind(id).execute(&mut *c).await?;
            if from != to {
                event(c,a,"TicketStatusChanged",id,json!({"ticket_id":id,"organization_id":a.org_id,"old_status":from,"new_status":to,"changed_by":a.user_id,"changed_at":chrono::Utc::now()})).await?;
            }
        }
        "ticket.priority" => {
            let priority = args["priority"].as_str().unwrap_or("");
            if !["low", "normal", "high", "urgent"].contains(&priority) {
                return Err(AppError::BadRequest("Invalid priority".into()));
            }
            sqlx::query("UPDATE tickets SET priority=$1,updated_at=now() WHERE organization_id=$2 AND id=$3").bind(priority).bind(a.org_id).bind(id).execute(&mut *c).await?;
        }
        "agent.notify" => {
            event(
                c,
                a,
                "AgentAttentionRequested",
                id,
                json!({"source":"workflow"}),
            )
            .await?;
        }
        "csat.request" => {
            let status: TicketStatus =
                sqlx::query_scalar("SELECT status FROM tickets WHERE organization_id=$1 AND id=$2")
                    .bind(a.org_id)
                    .bind(id)
                    .fetch_one(&mut *c)
                    .await?;
            if !matches!(status, TicketStatus::Solved | TicketStatus::Closed) {
                return Err(AppError::BadRequest(
                    "Resolve the ticket before requesting feedback".into(),
                ));
            }
            sqlx::query("INSERT INTO messages(organization_id,ticket_id,author_id,body,idempotency_key) VALUES($1,$2,$3,'How did we do? Please rate this resolved conversation using the satisfaction controls in your portal.',$4)").bind(a.org_id).bind(id).bind(a.user_id).bind(Uuid::new_v4()).execute(&mut *c).await?;
            event(c, a, "FeedbackRequested", id, json!({"delivery":"portal"})).await?;
        }
        "account.lookup" => {
            let row:Value=sqlx::query_scalar("SELECT jsonb_build_object('name',u.full_name,'email',u.email) FROM users u JOIN tickets t ON t.created_by=u.id AND t.organization_id=u.organization_id WHERE t.organization_id=$1 AND t.id=$2")
                .bind(a.org_id).bind(id).fetch_one(&mut *c).await?;
            return Ok(row);
        }
        _ => {
            return Err(AppError::Forbidden(
                "External tool execution disabled".into(),
            ))
        }
    }
    audit(
        c,
        a,
        "tool.executed",
        id,
        json!({"action":action,"arguments":args}),
    )
    .await?;
    Ok(json!({"executed":true,"action":action}))
}
pub async fn approvals(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(e) FROM tool_executions e WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 100").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
pub struct Decision {
    pub approve: bool,
}
pub async fn decide(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Decision>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    let mut tx = s.db.begin().await?;
    let row: Value = sqlx::query_scalar(
        "SELECT to_jsonb(e) FROM tool_executions e WHERE organization_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(a.org_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Action not found".into()))?;
    if row["state"] != "pending" {
        return Err(AppError::BadRequest("Action already decided".into()));
    }
    // Approval is NOT provider execution. Deliberately fail closed.
    let review = row["action"] == "human.review";
    let state = if !p.approve {
        "rejected"
    } else if review {
        "executed"
    } else {
        "blocked"
    };
    sqlx::query("UPDATE tool_executions SET state=$1,approved_by=$2,updated_at=now(),result=$3 WHERE organization_id=$4 AND id=$5")
        .bind(state).bind(a.user_id).bind(json!({"approved":p.approve,"executed":review && p.approve,"reason":if review {"Human review recorded; workflow does not automatically resume"}else{"External adapter not configured"}})).bind(a.org_id).bind(id).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &a,
        "tool.decided",
        id,
        json!({"approved":p.approve,"state":state}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"state":state,"executed":review && p.approve})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub name: String,
    pub definition: Definition,
}
pub async fn versions(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM workflows WHERE organization_id=$1 AND id=$2)",
    )
    .bind(a.org_id)
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    if !exists {
        return Err(AppError::NotFound("Workflow not found".into()));
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(v) FROM workflow_versions v WHERE organization_id=$1 AND workflow_id=$2 ORDER BY version DESC LIMIT 50").bind(a.org_id).bind(id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
pub struct Rollback {
    pub version: i32,
}
pub async fn rollback(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Rollback>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    let mut tx = s.db.begin().await?;
    let definition:Value=sqlx::query_scalar("SELECT definition FROM workflow_versions WHERE organization_id=$1 AND workflow_id=$2 AND version=$3").bind(a.org_id).bind(id).bind(p.version).fetch_optional(&mut *tx).await?.ok_or_else(||AppError::NotFound("Workflow version not found".into()))?;
    policy::validate(&serde_json::from_value(definition.clone())?)?;
    sqlx::query(
        "UPDATE workflows SET draft=$1,updated_at=now() WHERE organization_id=$2 AND id=$3",
    )
    .bind(&definition)
    .bind(a.org_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        &a,
        "workflow.draft_restored",
        id,
        json!({"source_version":p.version}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"draft":definition,"restored_version":p.version,"published_changed":false}),
    ))
}
pub async fn workflows(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(w) FROM workflows w WHERE organization_id=$1 ORDER BY updated_at DESC LIMIT 100").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn save_workflow(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Workflow>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    policy::validate(&p.definition)?;
    let name = text(&p.name, 160)?;
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM workflows WHERE organization_id=$1")
        .bind(a.org_id)
        .fetch_one(&mut *tx)
        .await?;
    if count >= 25 {
        return Err(AppError::BadRequest(
            "Trial workflow quota reached (25)".into(),
        ));
    }
    let row:Value=sqlx::query_scalar("INSERT INTO workflows(organization_id,name,draft) VALUES($1,$2,$3) RETURNING to_jsonb(workflows)").bind(a.org_id).bind(name).bind(serde_json::to_value(&p.definition)?).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(row))
}
pub async fn edit_workflow(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Workflow>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    policy::validate(&p.definition)?;
    let row:Value=sqlx::query_scalar("UPDATE workflows SET name=$1,draft=$2,updated_at=now() WHERE organization_id=$3 AND id=$4 RETURNING to_jsonb(workflows)")
        .bind(text(&p.name,160)?).bind(serde_json::to_value(&p.definition)?).bind(a.org_id).bind(id).fetch_optional(&s.db).await?.ok_or_else(||AppError::NotFound("Workflow not found".into()))?;
    Ok(Json(row))
}
pub async fn publish_workflow(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    let mut tx = s.db.begin().await?;
    let draft: Value = sqlx::query_scalar(
        "SELECT draft FROM workflows WHERE organization_id=$1 AND id=$2 FOR UPDATE",
    )
    .bind(a.org_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Workflow not found".into()))?;
    policy::validate(&serde_json::from_value(draft.clone())?)?;
    let version:i32=sqlx::query_scalar("UPDATE workflows SET published=draft,version=version+1,updated_at=now() WHERE organization_id=$1 AND id=$2 RETURNING version").bind(a.org_id).bind(id).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO workflow_versions(organization_id,workflow_id,version,definition) VALUES($1,$2,$3,$4)").bind(a.org_id).bind(id).bind(version).bind(draft).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &a,
        "workflow.published",
        id,
        json!({"version":version}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"version":version})))
}
#[derive(Deserialize)]
pub struct Run {
    pub ticket_id: Uuid,
    pub simulation: bool,
    pub idempotency_key: Uuid,
}
pub async fn run_workflow(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Run>,
) -> AppResult<Json<Value>> {
    staff(&a)?;
    if !p.simulation {
        admin(&a)?;
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    if let Some(row) = sqlx::query_scalar::<_, Value>(
        "SELECT to_jsonb(r) FROM workflow_runs r WHERE organization_id=$1 AND idempotency_key=$2",
    )
    .bind(a.org_id)
    .bind(p.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if row["workflow_id"] != json!(id)
            || row["ticket_id"] != json!(p.ticket_id)
            || row["simulation"] != p.simulation
        {
            return Err(AppError::BadRequest("Idempotency key conflict".into()));
        }
        return Ok(Json(row));
    }
    let quota:i64=sqlx::query_scalar("SELECT count(*) FROM workflow_runs WHERE organization_id=$1 AND created_at>date_trunc('day',now())").bind(a.org_id).fetch_one(&mut *tx).await?;
    if quota >= 500 {
        return Err(AppError::BadRequest(
            "Daily workflow quota reached (500)".into(),
        ));
    }
    ticket(&mut tx, &a, p.ticket_id).await?;
    let row = sqlx::query(
        "SELECT draft,published,version FROM workflows WHERE organization_id=$1 AND id=$2",
    )
    .bind(a.org_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Workflow not found".into()))?;
    let definition: Value = if p.simulation {
        row.get("draft")
    } else {
        row.get::<Option<Value>, _>("published")
            .ok_or_else(|| AppError::BadRequest("Publish before executing".into()))?
    };
    let definition: Definition = serde_json::from_value(definition)?;
    policy::validate(&definition)?;
    let (status, trace) =
        crate::workflow_engine::execute(&mut tx, &a, p.ticket_id, &definition, p.simulation)
            .await?;
    let run:Value=sqlx::query_scalar("INSERT INTO workflow_runs(organization_id,workflow_id,ticket_id,version,simulation,state,trace,idempotency_key) VALUES($1,$2,$3,$4,$5,$6,$7,$8) RETURNING to_jsonb(workflow_runs)")
        .bind(a.org_id).bind(id).bind(p.ticket_id).bind(row.get::<i32,_>("version")).bind(p.simulation).bind(&status).bind(json!(trace)).bind(p.idempotency_key).fetch_one(&mut *tx).await?;
    audit(
        &mut tx,
        &a,
        "workflow.run",
        id,
        json!({"simulation":p.simulation,"state":status}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(run))
}
pub async fn runs(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(r) FROM workflow_runs r WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 100").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn analytics(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let statuses:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('status',status,'count',count(*)) FROM tickets WHERE organization_id=$1 GROUP BY status").bind(a.org_id).fetch_all(&s.db).await?;
    let totals:Value=sqlx::query_scalar("SELECT jsonb_build_object('total',count(*),'backlog',count(*) FILTER(WHERE status NOT IN ('solved','closed')),'resolved',count(*) FILTER(WHERE status IN ('solved','closed')),'over_24h',count(*) FILTER(WHERE status NOT IN ('solved','closed') AND created_at<now()-interval '24 hours'),'unassigned',count(*) FILTER(WHERE assigned_to IS NULL AND status NOT IN ('solved','closed'))) FROM tickets WHERE organization_id=$1").bind(a.org_id).fetch_one(&s.db).await?;
    let daily:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('date',d::date,'count',(SELECT count(*) FROM tickets WHERE organization_id=$1 AND created_at>=d AND created_at<d+interval '1 day')) FROM generate_series(date_trunc('day',now())-interval '13 days',date_trunc('day',now()),interval '1 day') d").bind(a.org_id).fetch_all(&s.db).await?;
    let ai:Value=sqlx::query_scalar("SELECT jsonb_build_object('proposals',count(*),'abstained',count(*) FILTER(WHERE abstained),'cited',count(*) FILTER(WHERE NOT abstained),'model_cost_usd',0,'mode','extractive; no paid inference') FROM ai_proposals WHERE organization_id=$1").bind(a.org_id).fetch_one(&s.db).await?;
    let response:Option<f64>=sqlx::query_scalar("SELECT avg(extract(epoch FROM (first_reply-created_at))/60)::float8 FROM (SELECT t.created_at,(SELECT min(m.created_at) FROM messages m JOIN users u ON u.id=m.author_id AND u.organization_id=m.organization_id WHERE m.ticket_id=t.id AND m.organization_id=t.organization_id AND NOT m.internal AND u.role IN ('admin','agent')) first_reply FROM tickets t WHERE organization_id=$1) r").bind(a.org_id).fetch_one(&s.db).await?;
    let csat: Option<f64> =
        sqlx::query_scalar("SELECT avg(score)::float8 FROM csat WHERE organization_id=$1")
            .bind(a.org_id)
            .fetch_one(&s.db)
            .await?;
    Ok(Json(
        json!({"totals":totals,"statuses":statuses,"daily":daily,"ai":ai,"first_response_minutes":response,"csat_average":csat,"definitions":{"backlog":"Tickets outside solved/closed","over_24h":"Open backlog older than 24 hours, not a configured SLA","resolved":"Agent-set solved/closed; not independently verified resolution","first_response_minutes":"Mean minutes to first public agent reply; unanswered tickets excluded","model_cost":"Extractive retrieval has zero model API cost; infrastructure excluded"}}),
    ))
}
pub async fn audit_log(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    admin(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(e) FROM audit_events e WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 250").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
pub async fn settings(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    staff(&a)?;
    let row:Value=sqlx::query_scalar("SELECT jsonb_build_object('name',name,'slug',slug,'settings',settings) FROM organizations WHERE id=$1").bind(a.org_id).fetch_one(&s.db).await?;
    Ok(Json(row))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub name: String,
    pub reply_tone: String,
    pub onboarding_complete: bool,
}
pub async fn update_settings(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Settings>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    let name = text(&p.name, 100)?;
    if !["professional", "friendly", "concise"].contains(&p.reply_tone.as_str()) {
        return Err(AppError::BadRequest("Invalid reply tone".into()));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query(
        "UPDATE organizations SET name=$1,settings=settings || $2,updated_at=now() WHERE id=$3",
    )
    .bind(name)
    .bind(json!({"reply_tone":p.reply_tone,"onboarding_complete":p.onboarding_complete}))
    .bind(a.org_id)
    .execute(&mut *tx)
    .await?;
    audit(&mut tx, &a, "organization.updated", a.org_id, json!({})).await?;
    tx.commit().await?;
    Ok(Json(json!({"saved":true})))
}
pub async fn integrations(a: AuthUser) -> AppResult<Json<Value>> {
    staff(&a)?;
    Ok(Json(json!([
        {"id":"portal","name":"Customer portal","status":"active","detail":"Authenticated messages; customers see only their conversations."},
        {"id":"email","name":"Email","status":"not_configured","detail":"No outbound email provider connected. Portal replies are not email delivery."},
        {"id":"whatsapp","name":"WhatsApp Business","status":"not_configured","detail":"Official provider credentials and approval required."},
        {"id":"model","name":"Grounded copilot","status":"local","detail":"Postgres full-text retrieval; extractive evidence; no model API charges."},
        {"id":"stripe","name":"Payments","status":"disabled","detail":"No checkout or charges. Trial limits are local and enforced server-side."}
    ])))
}
pub async fn billing(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    admin(&a)?;
    let usage:Value=sqlx::query_scalar("SELECT jsonb_build_object('documents',(SELECT count(*) FROM knowledge_documents WHERE organization_id=$1 AND status='ready'),'drafts_today',(SELECT count(*) FROM ai_proposals WHERE organization_id=$1 AND created_at>date_trunc('day',now())),'workflows',(SELECT count(*) FROM workflows WHERE organization_id=$1),'runs_today',(SELECT count(*) FROM workflow_runs WHERE organization_id=$1 AND created_at>date_trunc('day',now())))").bind(a.org_id).fetch_one(&s.db).await?;
    Ok(Json(
        json!({"plan":"Local trial","provider":"disabled","charges_enabled":false,"usage":usage,"limits":{"documents":100,"drafts_today":200,"workflows":25,"runs_today":500},"invoices":[]}),
    ))
}
#[derive(Deserialize)]
pub struct Invite {
    pub email: String,
    pub role: Role,
}
fn digest(token: &str) -> String {
    super::identity::digest(token)
}
pub async fn invite(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Invite>,
) -> AppResult<Json<Value>> {
    admin(&a)?;
    let email = text(&p.email, 254)?.to_lowercase();
    super::identity::validate_beta_email(&email)?;
    if p.role == Role::Admin || !email.contains('@') {
        return Err(AppError::BadRequest(
            "Invite agent/customer with a valid email".into(),
        ));
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    if std::env::var("BETA_MODE").as_deref() == Ok("true") {
        let count:i64=sqlx::query_scalar("SELECT (SELECT count(*) FROM users WHERE organization_id=$1)+(SELECT count(*) FROM invitations WHERE organization_id=$1 AND used_at IS NULL AND expires_at>now())").bind(a.org_id).fetch_one(&mut *tx).await?;
        if count >= 20 {
            return Err(AppError::BadRequest(
                "Free beta limit: 20 members and pending invitations combined".into(),
            ));
        }
    }

    sqlx::query(
        "INSERT INTO invitations(organization_id,email,role,token_hash) VALUES($1,$2,$3,$4)",
    )
    .bind(a.org_id)
    .bind(email)
    .bind(p.role)
    .bind(digest(&token))
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        &a,
        "invitation.created",
        a.org_id,
        json!({"role":p.role}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"token":token,"expires_in_hours":48,"delivery":"manual_share; no email sent"}),
    ))
}
#[derive(Deserialize)]
pub struct Accept {
    pub token: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
}
pub async fn accept_invite(
    State(s): State<AppState>,
    Json(p): Json<Accept>,
) -> AppResult<(axum::http::HeaderMap, Json<Value>)> {
    super::identity::validate_beta_email(&p.email)?;
    if p.password.len() < 12 || p.password.len() > 256 {
        return Err(AppError::BadRequest(
            "Password requires 12–256 characters".into(),
        ));
    }
    let name = text(&p.full_name, 200)?;
    let hash = auth::hash_password(&p.password)?;
    let mut tx = s.db.begin().await?;
    let row=sqlx::query("UPDATE invitations SET used_at=now() WHERE token_hash=$1 AND lower(email)=$2 AND used_at IS NULL AND expires_at>now() RETURNING organization_id,role")
        .bind(digest(&p.token)).bind(p.email.trim().to_lowercase()).fetch_optional(&mut *tx).await?.ok_or_else(||AppError::BadRequest("Invalid or expired invitation".into()))?;
    let org_id: Uuid = row.get("organization_id");
    let role: Role = row.get("role");
    let id:Uuid=sqlx::query_scalar("INSERT INTO users(organization_id,email,password_hash,full_name,role) VALUES($1,$2,$3,$4,$5) RETURNING id")
        .bind(org_id).bind(p.email.trim().to_lowercase()).bind(hash).bind(name).bind(role).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET missing".into()))?;
    let token = auth::create_jwt(id, org_id, role, &secret)?;
    Ok(auth::session_response(
        &token,
        json!({"token":token.clone(),"user_id":id,"org_id":org_id}),
    ))
}
#[derive(Deserialize)]
pub struct Score {
    pub score: i32,
}
pub async fn csat(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<Score>,
) -> AppResult<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let t = ticket(&mut tx, &a, id).await?;
    if t["created_by"] != json!(a.user_id)
        || !["solved", "closed"].contains(&t["status"].as_str().unwrap_or(""))
        || !(1..=5).contains(&p.score)
    {
        return Err(AppError::BadRequest(
            "Only the ticket creator may rate a resolved ticket (1–5)".into(),
        ));
    }
    sqlx::query("INSERT INTO csat(organization_id,ticket_id,customer_id,score) VALUES($1,$2,$3,$4) ON CONFLICT(organization_id,ticket_id) DO UPDATE SET score=excluded.score").bind(a.org_id).bind(id).bind(a.user_id).bind(p.score).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"saved":true})))
}
pub async fn public_help(
    State(s): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<Search>,
) -> AppResult<Json<Value>> {
    let org: Option<Uuid> = sqlx::query_scalar("SELECT id FROM organizations WHERE slug=$1")
        .bind(slug)
        .fetch_optional(&s.db)
        .await?;
    let org = org.ok_or_else(|| AppError::NotFound("Help center not found".into()))?;
    let docs:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'title',title,'content',content,'updated_at',updated_at) FROM knowledge_documents WHERE organization_id=$1 AND visibility='public' AND status='ready' AND (expires_at IS NULL OR expires_at>now()) AND (title ILIKE $2 OR content ILIKE $2) LIMIT 100").bind(org).bind(format!("%{}%",q.q.chars().take(200).collect::<String>())).fetch_all(&s.db).await?;
    Ok(Json(json!(docs)))
}
