use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::auth::{self, AuthUser};
use crate::error::{AppError, AppResult};
use crate::models::{
    AuthResponse, LoginRequest, MeResponse, Role, SignupRequest, UpdateUserRoleRequest,
};
use crate::AppState;

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Json<AuthResponse>> {
    let (org_id, assigned_role) = match payload.organization_id {
        // ---- Case 1: joining an EXISTING org ----
        Some(existing_org_id) => {
            let org_exists = sqlx::query!(
                "SELECT id FROM organizations WHERE id = $1",
                existing_org_id
            )
            .fetch_optional(&state.db)
            .await?
            .is_some();

            if !org_exists {
                return Err(AppError::BadRequest("Organization not found".to_string()));
            }

            // SECURITY: joining an existing org can never grant admin —
            // only an existing admin can promote someone.
            let role = match payload.role {
                Some(Role::Admin) => {
                    return Err(AppError::BadRequest(
                        "Cannot self-assign admin role".to_string(),
                    ));
                }
                Some(role) => role,
                None => Role::Customer,
            };

            (existing_org_id, role)
        }
        // ---- Case 2: creating a NEW org ----
        None => {
            let org_name = payload.organization_name.clone().ok_or_else(|| {
                AppError::BadRequest(
                    "organization_name is required to create a new organization".to_string(),
                )
            })?;

            let org_slug = org_name.to_lowercase().replace(' ', "-");

            let org = sqlx::query!(
                "INSERT INTO organizations (name, slug) VALUES ($1, $2) RETURNING id",
                org_name,
                org_slug
            )
            .fetch_one(&state.db)
            .await?;

            // The person creating a brand-new org is always its admin
            (org.id, Role::Admin)
        }
    };

    let password_hash = auth::hash_password(&payload.password)?;

    let user = sqlx::query!(
        r#"INSERT INTO users (organization_id, email, password_hash, full_name, role)
           VALUES ($1, $2, $3, $4, $5::user_role) RETURNING id"#,
        org_id,
        payload.email,
        password_hash,
        payload.full_name,
        assigned_role as Role
    )
    .fetch_one(&state.db)
    .await?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT_SECRET must be set".to_string()))?;
    let token = auth::create_jwt(user.id, org_id, assigned_role, &jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        org_id,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query!(
        r#"SELECT id, organization_id, password_hash, role as "role: Role"
           FROM users WHERE email = $1"#,
        payload.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let valid = auth::verify_password(&payload.password, &user.password_hash)?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT_SECRET must be set".to_string()))?;
    let token = auth::create_jwt(user.id, user.organization_id, user.role, &jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        org_id: user.organization_id,
    }))
}

pub async fn me(auth_user: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        user_id: auth_user.user_id,
        org_id: auth_user.org_id,
        role: auth_user.role,
    })
}

pub async fn update_user_role(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: only an admin may change anyone's role.
    if auth_user.role != Role::Admin {
        return Err(AppError::Forbidden(
            "Only admins can change user roles".to_string(),
        ));
    }

    // Layer 1: the target user must be in the SAME org as the caller.
    let target = sqlx::query!(
        r#"SELECT id, organization_id, role as "role: Role" FROM users WHERE id = $1"#,
        target_user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if target.organization_id != auth_user.org_id {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Edge case: prevent an org from being left with zero admins.
    if target.role == Role::Admin && payload.role != Role::Admin {
        let admin_count = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM users WHERE organization_id = $1 AND role = 'admin'"#,
            auth_user.org_id
        )
        .fetch_one(&state.db)
        .await?
        .count;

        if admin_count <= 1 {
            return Err(AppError::BadRequest(
                "Cannot demote the organization's last admin".to_string(),
            ));
        }
    }

    sqlx::query!(
        "UPDATE users SET role = $1::user_role WHERE id = $2",
        payload.role as Role,
        target_user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "user_id": target_user_id, "role": payload.role })))
}
