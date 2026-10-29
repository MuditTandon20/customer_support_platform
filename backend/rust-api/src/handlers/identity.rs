use crate::{
    auth,
    error::{AppError, AppResult},
    AppState,
};
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn digest(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn sandbox() -> bool {
    std::env::var("ENABLE_SANDBOX").as_deref() == Ok("true")
}
pub async fn queue_for_user(
    c: &mut sqlx::PgConnection,
    id: Uuid,
    org: Uuid,
    email: &str,
    kind: &str,
) -> AppResult<()> {
    if !sandbox() {
        return Ok(());
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    sqlx::query("INSERT INTO identity_tokens(user_id,kind,token_hash) VALUES($1,$2,$3)")
        .bind(id)
        .bind(kind)
        .bind(digest(&token))
        .execute(&mut *c)
        .await?;
    let base = std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    sqlx::query("INSERT INTO sandbox_mail(organization_id,recipient,subject,body,direction,idempotency_key) VALUES($1,$2,$3,$4,'identity',$5)").bind(org).bind(email).bind(format!("Sandbox {kind} link"))
        .bind(format!("{base}/{}?token={token}\nLocal sandbox only. No email delivered. Expires in one hour.",if kind=="reset"{"reset-password"}else{"verify-email"})).bind(Uuid::new_v4()).execute(c).await?;
    Ok(())
}
#[derive(Deserialize)]
pub struct Email {
    pub email: String,
}
pub async fn request_reset(
    State(s): State<AppState>,
    Json(p): Json<Email>,
) -> AppResult<Json<Value>> {
    if !sandbox() {
        return Err(AppError::BadRequest(
            "Email delivery is not configured; enable the local sandbox or configure a provider"
                .into(),
        ));
    }
    let mut tx = s.db.begin().await?;
    let row: Option<(Uuid, Uuid)> =
        sqlx::query_as("SELECT id,organization_id FROM users WHERE lower(email)=$1")
            .bind(p.email.trim().to_lowercase())
            .fetch_optional(&mut *tx)
            .await?;
    if let Some((id, org)) = row {
        queue_for_user(&mut tx, id, org, p.email.trim(), "reset").await?;
    }
    tx.commit().await?;
    Ok(Json(
        json!({"accepted":true,"delivery":"sandbox_only","message":"If the account exists, a link was placed in its organization's local sandbox mailbox. No email was sent."}),
    ))
}
#[derive(Deserialize)]
pub struct Reset {
    pub token: String,
    pub password: String,
}
pub async fn reset(State(s): State<AppState>, Json(p): Json<Reset>) -> AppResult<Json<Value>> {
    if p.password.len() < 12 || p.password.len() > 256 {
        return Err(AppError::BadRequest(
            "Password requires 12–256 characters".into(),
        ));
    }
    let hash = auth::hash_password(&p.password)?;
    let mut tx = s.db.begin().await?;
    let id:Uuid=sqlx::query_scalar("UPDATE identity_tokens SET used_at=now() WHERE token_hash=$1 AND kind='reset' AND used_at IS NULL AND expires_at>now() RETURNING user_id").bind(digest(&p.token)).fetch_optional(&mut *tx).await?.ok_or_else(||AppError::BadRequest("Invalid or expired reset token".into()))?;
    sqlx::query("UPDATE users SET password_hash=$1,token_version=token_version+1 WHERE id=$2")
        .bind(hash)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE identity_tokens SET used_at=now() WHERE user_id=$1 AND kind='reset' AND used_at IS NULL").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"reset":true,"sessions_revoked":true})))
}
#[derive(Deserialize)]
pub struct Token {
    pub token: String,
}
pub async fn verify(State(s): State<AppState>, Json(p): Json<Token>) -> AppResult<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let id:Uuid=sqlx::query_scalar("UPDATE identity_tokens SET used_at=now() WHERE token_hash=$1 AND kind='verify' AND used_at IS NULL AND expires_at>now() RETURNING user_id").bind(digest(&p.token)).fetch_optional(&mut *tx).await?.ok_or_else(||AppError::BadRequest("Invalid or expired verification token".into()))?;
    sqlx::query("UPDATE users SET email_verified=true WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"verified":true})))
}

pub fn validate_beta_email(email: &str) -> AppResult<()> {
    if std::env::var("BETA_MODE").as_deref() == Ok("true")
        && !email.trim().to_lowercase().ends_with(".test")
    {
        return Err(AppError::BadRequest(
            "This testing beta accepts synthetic .test email addresses only".into(),
        ));
    }
    Ok(())
}
