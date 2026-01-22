use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use chrono::Utc;

use crate::{config::Config, errors::{AppError, AppResult}, db::DbPool};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,           // Subject (user_id)
    pub username: String,   // Username
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub token_type: String, // "access" or "refresh"
}

impl Claims {
    pub fn new_access_token(user_id: i32, username: String, expiry: i64) -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        Self {
            sub: user_id,
            username,
            exp: (now as i64 + expiry) as usize,
            iat: now,
            token_type: "access".to_string(),
        }
    }

    pub fn new_refresh_token(user_id: i32, username: String, expiry: i64) -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        Self {
            sub: user_id,
            username,
            exp: (now as i64 + expiry) as usize,
            iat: now,
            token_type: "refresh".to_string(),
        }
    }
}

pub fn generate_jwt(claims: &Claims, secret: &str) -> AppResult<String> {
    let token = encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub async fn auth_middleware(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Authentication(
            "Invalid authorization header format".to_string(),
        ));
    }

    let token = auth_header.trim_start_matches("Bearer ");

    // Try to validate as JWT first (for regular user authentication)
    let claims = match verify_jwt(token, &config.jwt_secret) {
        Ok(jwt_claims) => {
            // Valid JWT token
            if jwt_claims.token_type != "access" {
                return Err(AppError::Authentication("Invalid token type".to_string()));
            }
            tracing::debug!("Authenticated with JWT token for user_id={}", jwt_claims.sub);
            jwt_claims
        }
        Err(_) => {
            // Not a valid JWT, try OAuth access token
            tracing::debug!("Token is not a valid JWT, checking OAuth access tokens");

            match verify_oauth_token(&pool, token).await {
                Ok(oauth_claims) => {
                    tracing::info!("Authenticated with OAuth access token for user_id={}", oauth_claims.sub);
                    oauth_claims
                }
                Err(e) => {
                    tracing::error!("Authentication failed: not a valid JWT or OAuth token: {}", e);
                    return Err(AppError::Authentication("Invalid token".to_string()));
                }
            }
        }
    };

    // Insert claims into request extensions so handlers can access them
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Verify OAuth access token by looking it up in the database
async fn verify_oauth_token(pool: &PgPool, token: &str) -> AppResult<Claims> {
    // Look up token in database
    let row = sqlx::query(
        "SELECT user_id, expires_at FROM oauth_tokens WHERE token = $1"
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Authentication("OAuth token not found".to_string()))?;

    let user_id: i32 = row.get("user_id");
    let expires_at: chrono::DateTime<Utc> = row.get("expires_at");

    // Check if token has expired
    if expires_at < Utc::now() {
        return Err(AppError::Authentication("OAuth token expired".to_string()));
    }

    // Get user info
    let user = sqlx::query("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let username: String = user.get("username");

    // Create claims from OAuth token
    let exp = expires_at.timestamp() as usize;
    let now = Utc::now().timestamp() as usize;

    Ok(Claims {
        sub: user_id,
        username,
        exp,
        iat: now,
        token_type: "access".to_string(),
    })
}

// Custom extractor for Claims that works with FromRequestParts
// This allows handlers to extract Claims without consuming the full Request
#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::Authentication("No claims found".to_string()))
    }
}
