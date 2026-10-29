use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::Role,
    AppState,
};
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
fn allowed(a: &AuthUser) -> AppResult<()> {
    if a.role != Role::Admin || !super::identity::sandbox() {
        Err(AppError::Forbidden(
            "Local sandbox requires an administrator and ENABLE_SANDBOX=true".into(),
        ))
    } else {
        Ok(())
    }
}
pub async fn mailbox(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<Value>> {
    allowed(&a)?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(m) FROM sandbox_mail m WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 100").bind(a.org_id).fetch_all(&s.db).await?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inbound {
    pub customer_email: String,
    pub subject: String,
    pub body: String,
    pub ticket_id: Option<Uuid>,
    pub idempotency_key: Uuid,
}
pub async fn inbound(
    a: AuthUser,
    State(s): State<AppState>,
    Json(p): Json<Inbound>,
) -> AppResult<Json<Value>> {
    allowed(&a)?;
    if p.subject.trim().is_empty()
        || p.subject.len() > 200
        || p.body.trim().is_empty()
        || p.body.len() > 20000
    {
        return Err(AppError::BadRequest(
            "Subject (1–200) and body (1–20000) required".into(),
        ));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    if let Some(previous) = sqlx::query_scalar::<_, Value>(
        "SELECT to_jsonb(m) FROM sandbox_mail m WHERE organization_id=$1 AND idempotency_key=$2",
    )
    .bind(a.org_id)
    .bind(p.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if previous["body"] != p.body
            || previous["subject"] != p.subject
            || previous["recipient"] != p.customer_email
            || p.ticket_id
                .is_some_and(|id| previous["ticket_id"] != json!(id))
        {
            return Err(AppError::BadRequest("Idempotency key conflict".into()));
        }
        return Ok(Json(previous));
    }
    let customer: Uuid = sqlx::query_scalar(
        "SELECT id FROM users WHERE organization_id=$1 AND lower(email)=$2 AND role='customer'",
    )
    .bind(a.org_id)
    .bind(p.customer_email.trim().to_lowercase())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            "Invite and register this customer before simulating inbound email".into(),
        )
    })?;
    let id = if let Some(id) = p.ticket_id {
        let found:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tickets WHERE id=$1 AND organization_id=$2 AND created_by=$3 AND channel='email_sandbox' AND status!='closed')").bind(id).bind(a.org_id).bind(customer).fetch_one(&mut *tx).await?;
        if !found {
            return Err(AppError::NotFound("Sandbox conversation not found".into()));
        }
        sqlx::query("INSERT INTO messages(organization_id,ticket_id,author_id,body,idempotency_key) VALUES($1,$2,$3,$4,$5)").bind(a.org_id).bind(id).bind(customer).bind(&p.body).bind(p.idempotency_key).execute(&mut *tx).await?;
        id
    } else {
        sqlx::query_scalar("INSERT INTO tickets(organization_id,created_by,subject,description,channel) VALUES($1,$2,$3,$4,'email_sandbox') RETURNING id").bind(a.org_id).bind(customer).bind(&p.subject).bind(&p.body).fetch_one(&mut *tx).await?
    };
    sqlx::query("UPDATE tickets SET updated_at=now() WHERE id=$1 AND organization_id=$2")
        .bind(id)
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let mail:Value=sqlx::query_scalar("INSERT INTO sandbox_mail(organization_id,ticket_id,recipient,subject,body,direction,idempotency_key) VALUES($1,$2,$3,$4,$5,'inbound',$6) RETURNING to_jsonb(sandbox_mail)").bind(a.org_id).bind(id).bind(p.customer_email).bind(p.subject).bind(p.body).bind(p.idempotency_key).fetch_one(&mut *tx).await?;
    super::platform::audit(
        &mut tx,
        &a,
        "sandbox.email_received",
        id,
        json!({"mode":"sandbox_only"}),
    )
    .await?;
    super::platform::event(
        &mut tx,
        &a,
        "MessageCreated",
        id,
        json!({"channel":"email_sandbox"}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(mail))
}
