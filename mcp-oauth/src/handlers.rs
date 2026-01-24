//! OAuth 2.0 endpoint handlers
//!
//! This module provides the core OAuth 2.0 handlers for the authorization
//! code grant flow (RFC 6749 Section 4.1).

use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{
    error::{OAuthError, OAuthResult},
    storage::{AuthCodeStorage, ClientValidator, TokenStorage, UserProvider},
    types::{
        AuthorizeRequest, AuthorizeResponse, ClientCredentials, OAuthConfig, StoredAuthCode,
        StoredToken, TokenRequest, TokenResponse,
    },
};

/// State container for OAuth handlers
///
/// This struct holds references to all the components needed by the OAuth handlers.
pub struct OAuthState<S> {
    /// Storage backend for auth codes, tokens, and user lookup
    pub storage: Arc<S>,
    /// OAuth configuration
    pub config: OAuthConfig,
}

impl<S> OAuthState<S> {
    /// Create a new OAuth state
    pub fn new(storage: Arc<S>, config: OAuthConfig) -> Self {
        Self { storage, config }
    }
}

impl<S> Clone for OAuthState<S> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            config: self.config.clone(),
        }
    }
}

/// Context for the authorize handler
///
/// This contains information about the authenticated user making the request.
#[derive(Debug, Clone)]
pub struct AuthorizeContext {
    /// The user ID of the authenticated user
    pub user_id: String,
}

/// OAuth 2.0 Authorize endpoint handler
///
/// This generates an authorization code that can be exchanged for an access token.
/// The user must already be authenticated before calling this endpoint.
///
/// # Arguments
///
/// * `state` - The OAuth state containing storage and configuration
/// * `context` - Information about the authenticated user
/// * `params` - The authorization request parameters
///
/// # Returns
///
/// Returns the authorization response with the generated code, or an error.
pub async fn authorize<S>(
    State(state): State<OAuthState<S>>,
    context: AuthorizeContext,
    Query(params): Query<AuthorizeRequest>,
) -> OAuthResult<Json<AuthorizeResponse>>
where
    S: ClientValidator + AuthCodeStorage + Send + Sync,
{
    tracing::info!(
        user_id = %context.user_id,
        client_id = %params.client_id,
        redirect_uri = %params.redirect_uri,
        response_type = %params.response_type,
        "OAuth authorize request"
    );

    // Validate client_id
    if !state.storage.validate_client_id(&params.client_id).await? {
        tracing::error!(client_id = %params.client_id, "Invalid client_id");
        return Err(OAuthError::InvalidClient("Invalid client_id".to_string()));
    }

    // Validate redirect_uri
    if !state
        .storage
        .validate_redirect_uri(&params.client_id, &params.redirect_uri)
        .await?
    {
        tracing::error!(
            client_id = %params.client_id,
            redirect_uri = %params.redirect_uri,
            "Invalid redirect_uri"
        );
        return Err(OAuthError::InvalidRequest(
            "Invalid redirect_uri".to_string(),
        ));
    }

    // Validate response_type is "code"
    if params.response_type != "code" {
        tracing::error!(response_type = %params.response_type, "Invalid response_type");
        return Err(OAuthError::InvalidRequest(
            "Only response_type=code is supported".to_string(),
        ));
    }

    // Validate PKCE code_challenge_method if provided
    if let Some(method) = &params.code_challenge_method {
        if method != "S256" && method != "plain" {
            return Err(OAuthError::InvalidRequest(
                "Unsupported code_challenge_method".to_string(),
            ));
        }
    }

    // Generate authorization code
    let code = generate_code();
    let expires_at = Utc::now() + chrono::Duration::seconds(state.config.code_expiry_seconds);

    // Store authorization code
    let stored_code = StoredAuthCode {
        code: code.clone(),
        client_id: params.client_id.clone(),
        user_id: context.user_id.clone(),
        redirect_uri: params.redirect_uri.clone(),
        scope: params.scope.clone(),
        expires_at,
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: params.code_challenge_method.clone(),
        used_at: None,
    };

    state.storage.store_auth_code(stored_code).await?;

    tracing::info!(
        user_id = %context.user_id,
        client_id = %params.client_id,
        "Authorization code generated"
    );

    Ok(Json(AuthorizeResponse {
        code,
        state: params.state,
    }))
}

/// OAuth 2.0 Token endpoint handler
///
/// Exchanges an authorization code for an access token.
/// Supports both `client_secret_post` and `client_secret_basic` authentication methods.
///
/// # Arguments
///
/// * `state` - The OAuth state containing storage and configuration
/// * `headers` - HTTP headers (for Basic auth)
/// * `payload` - The token request form data
///
/// # Returns
///
/// Returns the token response with the access token, or an error.
pub async fn token<S>(
    State(state): State<OAuthState<S>>,
    headers: HeaderMap,
    Form(payload): Form<TokenRequest>,
) -> OAuthResult<(StatusCode, Json<TokenResponse>)>
where
    S: ClientValidator + AuthCodeStorage + TokenStorage + UserProvider + Send + Sync,
{
    tracing::info!(
        client_id = %payload.client_id,
        grant_type = %payload.grant_type,
        redirect_uri = %payload.redirect_uri,
        "OAuth token request"
    );

    // Extract client_secret from form body or Authorization header
    let client_secret = extract_client_secret(&payload, &headers)?;

    // Verify client credentials
    let credentials = ClientCredentials {
        client_id: payload.client_id.clone(),
        client_secret,
    };

    if !state
        .storage
        .validate_client_credentials(&credentials)
        .await?
    {
        tracing::error!(client_id = %payload.client_id, "Invalid client credentials");
        return Err(OAuthError::InvalidClient(
            "Invalid client credentials".to_string(),
        ));
    }

    // Verify grant_type
    if payload.grant_type != "authorization_code" {
        tracing::error!(grant_type = %payload.grant_type, "Invalid grant_type");
        return Err(OAuthError::UnsupportedGrantType(
            "Only grant_type=authorization_code is supported".to_string(),
        ));
    }

    // Look up authorization code
    let stored_code = state
        .storage
        .get_auth_code(&payload.code)
        .await?
        .ok_or_else(|| {
            tracing::error!(code = %payload.code, "Authorization code not found");
            OAuthError::InvalidGrant("Authorization code not found".to_string())
        })?;

    // Validate the code belongs to the client
    if stored_code.client_id != payload.client_id {
        tracing::error!("Code client_id mismatch");
        return Err(OAuthError::InvalidGrant(
            "Authorization code not found".to_string(),
        ));
    }

    // Check if code has expired
    if stored_code.expires_at < Utc::now() {
        tracing::error!(
            expires_at = %stored_code.expires_at,
            now = %Utc::now(),
            "Authorization code expired"
        );
        return Err(OAuthError::InvalidGrant(
            "Authorization code expired".to_string(),
        ));
    }

    // Check if code was already used
    if stored_code.used_at.is_some() {
        tracing::error!("Authorization code already used");
        return Err(OAuthError::InvalidGrant(
            "Authorization code already used".to_string(),
        ));
    }

    // Check redirect_uri matches
    if stored_code.redirect_uri != payload.redirect_uri {
        tracing::error!(
            expected = %stored_code.redirect_uri,
            got = %payload.redirect_uri,
            "Redirect URI mismatch"
        );
        return Err(OAuthError::InvalidGrant("Redirect URI mismatch".to_string()));
    }

    // Verify PKCE code_verifier if code_challenge was provided
    if let Some(code_challenge) = &stored_code.code_challenge {
        let code_verifier = payload.code_verifier.as_ref().ok_or_else(|| {
            OAuthError::InvalidGrant("code_verifier required for PKCE".to_string())
        })?;

        let method = stored_code
            .code_challenge_method
            .as_deref()
            .unwrap_or("plain");

        let computed_challenge = match method {
            "S256" => {
                let mut hasher = Sha256::new();
                hasher.update(code_verifier.as_bytes());
                let hash = hasher.finalize();
                general_purpose::URL_SAFE_NO_PAD.encode(hash)
            }
            "plain" => code_verifier.clone(),
            _ => {
                return Err(OAuthError::InvalidGrant(
                    "Unsupported code_challenge_method".to_string(),
                ))
            }
        };

        if &computed_challenge != code_challenge {
            tracing::error!("PKCE verification failed");
            return Err(OAuthError::InvalidGrant(
                "PKCE verification failed".to_string(),
            ));
        }
    }

    // Mark code as used
    state.storage.mark_code_used(&payload.code).await?;

    // Generate access token
    let access_token = generate_token();
    let token_expires_at =
        Utc::now() + chrono::Duration::seconds(state.config.token_expiry_seconds);
    let scope_str = stored_code.scope.unwrap_or_else(|| "all".to_string());

    // Store access token
    let stored_token = StoredToken {
        token: access_token.clone(),
        client_id: payload.client_id.clone(),
        user_id: stored_code.user_id.clone(),
        scope: scope_str.clone(),
        expires_at: token_expires_at,
        last_used_at: None,
        revoked_at: None,
    };

    state.storage.store_token(stored_token).await?;

    tracing::info!(
        user_id = %stored_code.user_id,
        client_id = %payload.client_id,
        "Access token issued"
    );

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: state.config.token_expiry_seconds,
            scope: scope_str,
            refresh_token: None,
        }),
    ))
}

/// Extract client secret from request body or Authorization header
fn extract_client_secret(payload: &TokenRequest, headers: &HeaderMap) -> OAuthResult<String> {
    // Try form body first
    if let Some(secret) = &payload.client_secret {
        return Ok(secret.clone());
    }

    // Try Authorization header (Basic auth)
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        let auth_str = auth_header.to_str().map_err(|_| {
            OAuthError::InvalidRequest("Invalid Authorization header encoding".to_string())
        })?;

        if auth_str.starts_with("Basic ") {
            let encoded = &auth_str[6..];
            let decoded_bytes = general_purpose::STANDARD.decode(encoded).map_err(|_| {
                OAuthError::InvalidRequest("Invalid Authorization header encoding".to_string())
            })?;

            let decoded = String::from_utf8(decoded_bytes).map_err(|_| {
                OAuthError::InvalidRequest("Invalid Authorization header encoding".to_string())
            })?;

            // Format is "client_id:client_secret"
            if let Some((_id, secret)) = decoded.split_once(':') {
                tracing::debug!("Using client_secret from Basic Authorization header");
                return Ok(secret.to_string());
            } else {
                return Err(OAuthError::InvalidRequest(
                    "Invalid Basic auth format".to_string(),
                ));
            }
        }
    }

    Err(OAuthError::InvalidClient(
        "Missing client credentials".to_string(),
    ))
}

/// Generate a random authorization code (48 characters)
pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    // 36 bytes -> 48 base64 characters
    let random_bytes: Vec<u8> = (0..36).map(|_| rng.gen::<u8>()).collect();

    general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes)
}

/// Generate a random access token (96 characters)
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    // 72 bytes -> 96 base64 characters
    let random_bytes: Vec<u8> = (0..72).map(|_| rng.gen::<u8>()).collect();

    general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes)
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
