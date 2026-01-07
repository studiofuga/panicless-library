use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{config::Config, errors::{AppError, AppResult}};

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

    let claims = verify_jwt(token, &config.jwt_secret)?;

    if claims.token_type != "access" {
        return Err(AppError::Authentication(
            "Invalid token type".to_string(),
        ));
    }

    // Insert claims into request extensions so handlers can access them
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
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
