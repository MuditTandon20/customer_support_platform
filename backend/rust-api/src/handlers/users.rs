use crate::{
    auth::{self, AuthUser},
    error::{AppError, AppResult},
    models::*,
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::Row;
use uuid::Uuid;

pub async fn signup(
    State(s): State<AppState>,
    Json(p): Json<SignupRequest>,
) -> AppResult<(axum::http::HeaderMap, Json<AuthResponse>)> {
    if p.organization_id.is_some() {
        return Err(AppError::BadRequest(
            "Use an invitation to join an organization".into(),
        ));
    }
    let _legacy_requested_role = p.role; // Organization creators are always administrators.
    let email = p.email.trim().to_lowercase();
    let beta = std::env::var("BETA_MODE").as_deref() == Ok("true");
    if beta && !email.ends_with(".test") {
        return Err(AppError::BadRequest("This public beta accepts synthetic .test email addresses only. Do not enter real customer data.".into()));
    }
    let name = p.organization_name.as_deref().unwrap_or("").trim();
    if !email.contains('@') || email.len() > 254 || p.password.len() < 12 || p.password.len() > 256
    {
        return Err(AppError::BadRequest(
            "Valid email and password of 12-256 characters required".into(),
        ));
    }
    if name.is_empty()
        || name.len() > 100
        || p.full_name.trim().is_empty()
        || p.full_name.len() > 200
    {
        return Err(AppError::BadRequest(
            "Organization and full name required".into(),
        ));
    }
    let hash = auth::hash_password(&p.password)?;
    let mut tx = s.db.begin().await?;
    if beta {
        sqlx::query("SELECT pg_advisory_xact_lock(73192026)")
            .execute(&mut *tx)
            .await?;
        let max: i64 = std::env::var("MAX_ORGANIZATIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations")
            .fetch_one(&mut *tx)
            .await?;
        if count >= max {
            return Err(AppError::BadRequest("The free testing workspace limit has been reached. Try an existing test account or contact the operator.".into()));
        }
    }
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations(name,slug) VALUES($1,$2) RETURNING id")
            .bind(name)
            .bind(format!("workspace-{}", Uuid::new_v4()))
            .fetch_one(&mut *tx)
            .await?;
    let user_id:Uuid=sqlx::query_scalar("INSERT INTO users(organization_id,email,password_hash,full_name,role) VALUES($1,$2,$3,$4,'admin') RETURNING id").bind(org_id).bind(&email).bind(hash).bind(p.full_name.trim()).fetch_one(&mut *tx).await.map_err(|error| {
        if error.as_database_error().is_some_and(|db|db.is_unique_violation() && db.constraint()==Some("users_email_key")) {
            AppError::Conflict("An account with this email already exists. Sign in to your existing workspace, or use a different email to create a new organization.".into())
        } else {AppError::from(error)}
    })?;
    super::platform::audit(
        &mut tx,
        &AuthUser {
            user_id,
            org_id,
            role: Role::Admin,
        },
        "organization.created",
        org_id,
        serde_json::json!({}),
    )
    .await?;
    super::identity::queue_for_user(&mut tx, user_id, org_id, &email, "verify").await?;
    tx.commit().await?;
    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET missing".into()))?;
    let token = auth::create_jwt(user_id, org_id, Role::Admin, &secret)?;
    Ok(auth::session_response(
        &token,
        AuthResponse {
            token: token.clone(),
            user_id,
            org_id,
        },
    ))
}
pub async fn login(
    State(s): State<AppState>,
    Json(p): Json<LoginRequest>,
) -> AppResult<(axum::http::HeaderMap, Json<AuthResponse>)> {
    let row=sqlx::query("SELECT id,organization_id,password_hash,role,token_version FROM users WHERE lower(email)=$1").bind(p.email.trim().to_lowercase()).fetch_optional(&s.db).await?.ok_or_else(||AppError::Unauthorized("Invalid credentials".into()))?;
    if p.password.len() > 256
        || !auth::verify_password(&p.password, &row.get::<String, _>("password_hash"))?
    {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }
    let user_id = row.get("id");
    let org_id = row.get("organization_id");
    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET missing".into()))?;
    let token = auth::create_jwt_version(
        user_id,
        org_id,
        row.get("role"),
        row.get("token_version"),
        &secret,
    )?;
    Ok(auth::session_response(
        &token,
        AuthResponse {
            token: token.clone(),
            user_id,
            org_id,
        },
    ))
}
pub async fn me(a: AuthUser, State(s): State<AppState>) -> AppResult<Json<MeResponse>> {
    let row = sqlx::query("SELECT name,slug FROM organizations WHERE id=$1")
        .bind(a.org_id)
        .fetch_one(&s.db)
        .await?;
    Ok(Json(MeResponse {
        user_id: a.user_id,
        org_id: a.org_id,
        org_name: Some(row.get("name")),
        org_slug: Some(row.get("slug")),
        role: a.role,
    }))
}
pub async fn list_users(
    a: AuthUser,
    State(s): State<AppState>,
) -> AppResult<Json<Vec<UserSummary>>> {
    super::platform::staff(&a)?;
    let users = sqlx::query_as::<_, UserSummary>(
        "SELECT id,email,full_name,role FROM users WHERE organization_id=$1 ORDER BY full_name",
    )
    .bind(a.org_id)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(users))
}
pub async fn update_user_role(
    a: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(p): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if a.role != Role::Admin {
        return Err(AppError::Forbidden("Administrator required".into()));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM organizations WHERE id=$1 FOR NO KEY UPDATE")
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    let role: Role =
        sqlx::query_scalar("SELECT role FROM users WHERE id=$1 AND organization_id=$2")
            .bind(id)
            .bind(a.org_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    if role == Role::Admin && p.role != Role::Admin {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM users WHERE organization_id=$1 AND role='admin'",
        )
        .bind(a.org_id)
        .fetch_one(&mut *tx)
        .await?;
        if count <= 1 {
            return Err(AppError::BadRequest(
                "Cannot demote last administrator".into(),
            ));
        }
    }
    sqlx::query("UPDATE users SET role=$1 WHERE id=$2 AND organization_id=$3")
        .bind(p.role)
        .bind(id)
        .bind(a.org_id)
        .execute(&mut *tx)
        .await?;
    super::platform::audit(
        &mut tx,
        &a,
        "user.role_changed",
        id,
        serde_json::json!({"role":p.role}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(serde_json::json!({"user_id":id,"role":p.role})))
}
