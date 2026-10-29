use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Sqlx(sqlx::Error),
    NotFound(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
    Unauthorized(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Sqlx(e)
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::Sqlx(e) => {
                tracing::error!(error = %e, "Database operation failed");
                if e.as_database_error()
                    .is_some_and(|d| d.is_unique_violation())
                {
                    (
                        StatusCode::CONFLICT,
                        "Record or idempotency key already exists".into(),
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database operation failed".into(),
                    )
                }
            }
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            AppError::Internal(m) => {
                tracing::error!(error=%m, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service temporarily unavailable".into(),
                )
            }
        };

        (status, axum::Json(serde_json::json!({"error": msg}))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        eprintln!("AI copilot request failed: {e}");
        AppError::Internal("AI service is currently unavailable".to_string())
    }
}
