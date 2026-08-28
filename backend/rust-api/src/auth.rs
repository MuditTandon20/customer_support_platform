use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Role;

// Hash a plain password before storing it
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

// Verify a login attempt against the stored hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

// What data we embed inside the JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // user_id (subject)
    pub org_id: String,    // organization_id
    pub role: Role,        // embedded so we don't need a DB hit per request
    pub exp: usize,        // expiry timestamp
}

pub fn create_jwt(user_id: Uuid, org_id: Uuid, role: Role, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        org_id: org_id.to_string(),
        role,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
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
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract the Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        // 2. Strip the "Bearer " prefix
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".to_string()))?;

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

        Ok(AuthUser {
            user_id,
            org_id,
            role: claims.role,
        })
    }
}