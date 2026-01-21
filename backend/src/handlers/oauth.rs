use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use chrono::Utc;
use rand::Rng;
use jsonwebtoken::{encode, EncodingKey, Header};
use base64::{Engine as _, engine::general_purpose};

use crate::{
    config::Config,
    errors::AppError,
    middleware::Claims,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizeRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String, // "code"
    pub scope: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub grant_type: String, // "authorization_code"
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
    pub jwt_token: String, // JWT for accessing protected endpoints
}

/// OAuth2 Authorize endpoint
/// Generates an authorization code that Claude will use to request an access token
pub async fn authorize(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    claims: Claims,
    Query(params): Query<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, AppError> {
    // Verify client_id
    if params.client_id != config.oauth_client_id {
        return Err(AppError::Authentication("Invalid client_id".to_string()));
    }

    // Validate response_type is "code"
    if params.response_type != "code" {
        return Err(AppError::Validation(
            "Only response_type=code is supported".to_string(),
        ));
    }

    // Generate authorization code
    let code = generate_code();

    // Store authorization code in database (expires in 10 minutes)
    let expires_at = Utc::now() + chrono::Duration::minutes(10);

    sqlx::query(
        "INSERT INTO oauth_codes (code, client_id, user_id, redirect_uri, scope, expires_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&code)
    .bind(&params.client_id)
    .bind(claims.sub)
    .bind(&params.redirect_uri)
    .bind(&params.scope)
    .bind(expires_at)
    .execute(&pool)
    .await?;

    tracing::info!(
        "OAuth authorization code generated for user {} and client {}",
        claims.sub,
        params.client_id
    );

    Ok(Json(AuthorizeResponse {
        code,
        state: params.state,
    }))
}

/// OAuth2 Token endpoint
/// Exchanges authorization code for access token
pub async fn token(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(payload): Json<TokenRequest>,
) -> Result<(StatusCode, Json<TokenResponse>), AppError> {
    // Verify client credentials
    if payload.client_id != config.oauth_client_id || payload.client_secret != config.oauth_client_secret {
        tracing::warn!("Invalid OAuth client credentials attempt");
        return Err(AppError::Authentication("Invalid client credentials".to_string()));
    }

    // Verify grant_type
    if payload.grant_type != "authorization_code" {
        return Err(AppError::Validation(
            "Only grant_type=authorization_code is supported".to_string(),
        ));
    }

    // Look up authorization code
    let oauth_code = sqlx::query(
        "SELECT id, user_id, redirect_uri, scope, expires_at, used_at FROM oauth_codes WHERE code = $1 AND client_id = $2"
    )
    .bind(&payload.code)
    .bind(&payload.client_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Authentication("Authorization code not found".to_string()))?;

    let code_id: i32 = oauth_code.get("id");
    let user_id: i32 = oauth_code.get("user_id");
    let redirect_uri: String = oauth_code.get("redirect_uri");
    let scope: Option<String> = oauth_code.get("scope");
    let expires_at: chrono::DateTime<Utc> = oauth_code.get("expires_at");
    let used_at: Option<chrono::DateTime<Utc>> = oauth_code.get("used_at");

    // Check if code has expired
    if expires_at < Utc::now() {
        return Err(AppError::Validation("Authorization code expired".to_string()));
    }

    // Check if code was already used
    if used_at.is_some() {
        tracing::warn!("Attempt to reuse authorization code");
        return Err(AppError::Authentication("Authorization code already used".to_string()));
    }

    // Check redirect_uri matches
    if redirect_uri != payload.redirect_uri {
        return Err(AppError::Validation("Redirect URI mismatch".to_string()));
    }

    // Mark code as used
    sqlx::query(
        "UPDATE oauth_codes SET used_at = NOW() WHERE id = $1"
    )
    .bind(code_id)
    .execute(&pool)
    .await?;

    // Get user info for JWT
    let user = sqlx::query(
        "SELECT id, username FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    let username: String = user.get("username");

    // Generate access token
    let access_token = generate_token();
    let token_expires_at = Utc::now() + chrono::Duration::hours(24);
    let scope_str = scope.unwrap_or_else(|| "all".to_string());

    // Store access token
    sqlx::query(
        "INSERT INTO oauth_tokens (token, client_id, user_id, scope, expires_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&access_token)
    .bind(&payload.client_id)
    .bind(user_id)
    .bind(&scope_str)
    .bind(token_expires_at)
    .execute(&pool)
    .await?;

    // Generate JWT token for accessing protected endpoints
    let now = Utc::now();
    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username,
        exp,
        iat,
        token_type: "access".to_string(),
    };

    let jwt_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Validation("Failed to generate JWT".to_string()))?;

    tracing::info!(
        "OAuth access token issued for user {} and client {}",
        user_id,
        payload.client_id
    );

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 86400, // 24 hours
            scope: scope_str,
            jwt_token,
        }),
    ))
}

/// Generate a random authorization code
fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32)
        .map(|_| rng.gen::<u8>())
        .collect();

    general_purpose::STANDARD.encode(&random_bytes)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(48)
        .collect()
}

/// Generate a random access token
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..64)
        .map(|_| rng.gen::<u8>())
        .collect();

    general_purpose::STANDARD.encode(&random_bytes)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(96)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation() {
        let code1 = generate_code();
        let code2 = generate_code();

        assert_eq!(code1.len(), 48);
        assert_eq!(code2.len(), 48);
        assert_ne!(code1, code2);
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_token();
        let token2 = generate_token();

        assert_eq!(token1.len(), 96);
        assert_eq!(token2.len(), 96);
        assert_ne!(token1, token2);
    }
}
