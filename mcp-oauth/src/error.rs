//! OAuth error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// OAuth-specific error types
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("Invalid client: {0}")]
    InvalidClient(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid grant: {0}")]
    InvalidGrant(String),

    #[error("Unauthorized client: {0}")]
    UnauthorizedClient(String),

    #[error("Unsupported grant type: {0}")]
    UnsupportedGrantType(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Token generation error: {0}")]
    TokenGeneration(String),
}

/// OAuth error response as per RFC 6749
#[derive(Debug, Serialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let (status, error_code, description) = match &self {
            OAuthError::InvalidClient(msg) => {
                (StatusCode::UNAUTHORIZED, "invalid_client", msg.clone())
            }
            OAuthError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_request", msg.clone())
            }
            OAuthError::InvalidGrant(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_grant", msg.clone())
            }
            OAuthError::UnauthorizedClient(msg) => {
                (StatusCode::UNAUTHORIZED, "unauthorized_client", msg.clone())
            }
            OAuthError::UnsupportedGrantType(msg) => {
                (StatusCode::BAD_REQUEST, "unsupported_grant_type", msg.clone())
            }
            OAuthError::InvalidScope(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_scope", msg.clone())
            }
            OAuthError::ServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "server_error", msg.clone())
            }
            OAuthError::Storage(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "server_error", msg.clone())
            }
            OAuthError::TokenGeneration(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "server_error", msg.clone())
            }
        };

        let body = OAuthErrorResponse {
            error: error_code.to_string(),
            error_description: Some(description),
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias for OAuth operations
pub type OAuthResult<T> = Result<T, OAuthError>;
