use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Role;

// Hash a plain password before storing it
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes())?;
    Ok(hash.to_string())
}

// Verify a login attempt against the stored hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// What data we embed inside the JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user_id (subject)
    pub org_id: String, // organization_id
    pub role: Role,     // embedded so we don't need a DB hit per request
    pub exp: usize,     // expiry timestamp
    #[serde(default)]
    pub token_version: i64,
}

pub fn create_jwt(
    user_id: Uuid,
    org_id: Uuid,
    role: Role,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_jwt_version(user_id, org_id, role, 0, secret)
}
pub fn create_jwt_version(
    user_id: Uuid,
    org_id: Uuid,
    role: Role,
    token_version: i64,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    crate::jwt_crypto::install();
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        org_id: org_id.to_string(),
        role,
        exp: expiration,
        token_version,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    crate::jwt_crypto::install();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

// Represents an authenticated user, extracted from the JWT
pub struct AuthUser {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: Role,
}

#[async_trait]
impl FromRequestParts<crate::AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let bearer = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        let cookie = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.split(';')
                    .map(str::trim)
                    .find_map(|part| part.strip_prefix("aster_session="))
            });
        let token = bearer
            .or(cookie)
            .ok_or_else(|| AppError::Unauthorized("Authentication required".into()))?;

        // 3. Verify the JWT
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET not set".to_string()))?;

        let claims = verify_jwt(token, &jwt_secret)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        // 4. Parse the UUIDs out of the claims
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user_id in token".to_string()))?;
        let org_id = Uuid::parse_str(&claims.org_id)
            .map_err(|_| AppError::Unauthorized("Invalid org_id in token".to_string()))?;

        let (role, version): (Role, i64) = sqlx::query_as(
            "SELECT role, token_version FROM users WHERE id=$1 AND organization_id=$2",
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Session no longer valid".into()))?;
        if version != claims.token_version {
            return Err(AppError::Unauthorized("Session revoked".into()));
        }
        Ok(AuthUser {
            user_id,
            org_id,
            role,
        })
    }
}

pub fn cookie_header(token: &str, clear: bool) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    let secure = if std::env::var("COOKIE_SECURE").as_deref() == Ok("true") {
        "; Secure"
    } else {
        ""
    };
    let cookie = format!(
        "aster_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
        if clear { 0 } else { 86400 },
        secure
    );
    headers.insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    headers
}
pub fn session_response<T: serde::Serialize>(
    token: &str,
    value: T,
) -> (axum::http::HeaderMap, axum::Json<T>) {
    (cookie_header(token, false), axum::Json(value))
}
pub async fn logout() -> (axum::http::HeaderMap, axum::Json<serde_json::Value>) {
    (
        cookie_header("", true),
        axum::Json(serde_json::json!({"signed_out":true})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_hash_and_current_hash_roundtrip() {
        let legacy:&str="$argon2id$v=19$m=19456,t=2,p=1$aEfNkeza7DhBb9SysUl9Gw$Mmdr4IDoC1ohHkuKtvvTevoHhj+o3LSj6lpmvOUfP3c";
        assert!(verify_password("Demo-only-password-2026!", legacy).unwrap());
        assert!(!verify_password("incorrect", legacy).unwrap());
        let current = hash_password("Synthetic-roundtrip-2026!").unwrap();
        assert!(verify_password("Synthetic-roundtrip-2026!", &current).unwrap());
    }
    #[test]
    fn jwt_rejects_other_algorithms() {
        let secret = "synthetic-test-secret-at-least-32-characters";
        let user = Uuid::new_v4();
        let org = Uuid::new_v4();
        let good = create_jwt(user, org, Role::Admin, secret).unwrap();
        assert_eq!(verify_jwt(&good, secret).unwrap().sub, user.to_string());
        let claims = Claims {
            sub: user.to_string(),
            org_id: org.to_string(),
            role: Role::Admin,
            exp: (chrono::Utc::now().timestamp() + 60) as usize,
            token_version: 0,
        };
        let other = encode(
            &Header::new(jsonwebtoken::Algorithm::HS384),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        );
        assert!(other.is_err());
        let mut pieces: Vec<_> = good.split('.').collect();
        pieces[0] = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzM4NCJ9";
        assert!(verify_jwt(&pieces.join("."), secret).is_err());
        assert!(verify_jwt(&good, "different-secret").is_err());
        assert!(verify_jwt("not-a-token", secret).is_err());
    }
    #[test]
    fn jwt_legacy_hs256_fixture_and_expiration() {
        // Independently generated with Python stdlib hmac/hashlib, using the
        // original HS256 wire format and no token_version claim.
        let fixture = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDEiLCJvcmdfaWQiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDIiLCJyb2xlIjoiYWRtaW4iLCJleHAiOjQxMDI0NDQ4MDB9.hwWygY-ZQazCui6EcdcxoMYX9v6lquLks3yCExOhkIY";
        let secret = "legacy-synthetic-secret-at-least-32-characters";
        let claims = verify_jwt(fixture, secret).unwrap();
        assert_eq!(claims.token_version, 0);
        assert_eq!(claims.sub, "00000000-0000-0000-0000-000000000001");
        let mut tampered = fixture.to_string();
        let signature_start = tampered.rfind('.').unwrap() + 1;
        tampered.replace_range(signature_start..signature_start + 1, "A");
        assert!(verify_jwt(&tampered, secret).is_err());
        let expired = encode(
            &Header::default(),
            &Claims { exp: 1, ..claims },
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        assert!(verify_jwt(&expired, secret).is_err());
    }
}
