use crate::{auth::AuthUser, error::AppResult, AppState};
use serde::{de::DeserializeOwned, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub async fn record(
    c: &mut sqlx::PgConnection,
    a: &AuthUser,
    id: Uuid,
    operation: &str,
    mode: &str,
    status: &str,
    latency: i64,
) -> AppResult<()> {
    sqlx::query("INSERT INTO ai_activity_log(organization_id,ticket_id,operation,mode,status,latency_ms) VALUES($1,$2,$3,$4,$5,$6)").bind(a.org_id).bind(id).bind(operation).bind(mode).bind(status).bind(latency).execute(c).await?;
    Ok(())
}
pub async fn analyze<T: Serialize + DeserializeOwned, P: Serialize>(
    s: &AppState,
    a: &AuthUser,
    id: Uuid,
    operation: &str,
    payload: &P,
    fallback: T,
) -> AppResult<T> {
    let start = Instant::now();
    let attempt = async {
        s.http_client
            .post(format!("{}/copilot/{operation}", s.ai_copilot_url))
            .header(
                "X-Service-Token",
                std::env::var("AI_SERVICE_TOKEN").unwrap_or_default(),
            )
            .timeout(Duration::from_secs(6))
            .json(payload)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }
    .await;
    let (result, status, mode) = match attempt {
        Ok(value) => (value, "completed", "deterministic_service"),
        Err(error) => {
            tracing::warn!(operation,error=%error,"AI service unavailable; using bounded local extraction");
            (fallback, "fallback", "deterministic_fallback")
        }
    };
    record(
        &mut *s.db.acquire().await?,
        a,
        id,
        operation,
        mode,
        status,
        start.elapsed().as_millis() as i64,
    )
    .await?;
    Ok(result)
}
