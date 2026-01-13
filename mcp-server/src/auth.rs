use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// JWT Claims structure (must match backend)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,           // user_id
    pub username: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

/// Extract user_id from claims
impl Claims {
    pub fn user_id(&self) -> i32 {
        self.sub
    }
}

/// Verify and decode JWT token
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Invalid token: {}", e))?;

    // Verify it's an access token
    if token_data.claims.token_type != "access" {
        return Err("Token is not an access token".to_string());
    }

    Ok(token_data.claims)
}

/// Authentication middleware
pub async fn auth_middleware(
    axum::extract::State(config): axum::extract::State<Config>,
    mut request: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify JWT
    let claims = verify_jwt(token, &config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extractor for Claims from request
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn create_test_token(secret: &str) -> String {
        let claims = Claims {
            sub: 42,
            username: "testuser".to_string(),
            exp: 9999999999,
            iat: 1000000000,
            token_type: "access".to_string(),
        };

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key).unwrap()
    }

    #[test]
    fn test_verify_valid_token() {
        let secret = "test-secret";
        let token = create_test_token(secret);

        let result = verify_jwt(&token, secret);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_verify_invalid_secret() {
        let secret = "test-secret";
        let token = create_test_token(secret);

        let result = verify_jwt(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_refresh_token_rejected() {
        let claims = Claims {
            sub: 42,
            username: "testuser".to_string(),
            exp: 9999999999,
            iat: 1000000000,
            token_type: "refresh".to_string(), // Wrong type!
        };

        let secret = "test-secret";
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();

        let result = verify_jwt(&token, secret);
        assert!(result.is_err()); // Should reject refresh tokens
    }
}
