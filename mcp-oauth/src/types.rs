//! OAuth data types and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OAuth 2.0 Authorization Request (RFC 6749 Section 4.1.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeRequest {
    /// The client identifier
    pub client_id: String,
    /// The URI to redirect the user-agent after authorization
    pub redirect_uri: String,
    /// Must be "code" for authorization code flow
    pub response_type: String,
    /// The scope of the access request
    pub scope: Option<String>,
    /// Opaque value used to maintain state between request and callback
    pub state: Option<String>,
    /// PKCE code challenge (RFC 7636)
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (RFC 7636)
    pub code_challenge_method: Option<String>,
}

/// OAuth 2.0 Authorization Response (RFC 6749 Section 4.1.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    /// The authorization code
    pub code: String,
    /// The state parameter from the authorization request
    pub state: Option<String>,
}

/// OAuth 2.0 Token Request (RFC 6749 Section 4.1.3)
#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequest {
    /// The client identifier
    pub client_id: String,
    /// The client secret (may be in Authorization header instead)
    pub client_secret: Option<String>,
    /// The authorization code received from the authorization endpoint
    pub code: String,
    /// Must be "authorization_code"
    pub grant_type: String,
    /// The redirect URI used in the authorization request
    pub redirect_uri: String,
    /// PKCE code verifier (RFC 7636)
    pub code_verifier: Option<String>,
}

/// OAuth 2.0 Token Response (RFC 6749 Section 5.1)
#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    /// The access token
    pub access_token: String,
    /// The token type (always "Bearer")
    pub token_type: String,
    /// Token lifetime in seconds
    pub expires_in: i64,
    /// The scope of the access token
    pub scope: String,
    /// Optional refresh token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Stored authorization code data
#[derive(Debug, Clone)]
pub struct StoredAuthCode {
    /// The authorization code
    pub code: String,
    /// Client ID that requested the code
    pub client_id: String,
    /// User ID the code was granted for
    pub user_id: String,
    /// Redirect URI from the authorization request
    pub redirect_uri: String,
    /// Granted scope
    pub scope: Option<String>,
    /// When the code expires
    pub expires_at: DateTime<Utc>,
    /// PKCE code challenge (if provided)
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (if provided)
    pub code_challenge_method: Option<String>,
    /// When the code was used (None if not yet used)
    pub used_at: Option<DateTime<Utc>>,
}

/// Stored access token data
#[derive(Debug, Clone)]
pub struct StoredToken {
    /// The access token
    pub token: String,
    /// Client ID that the token was issued for
    pub client_id: String,
    /// User ID the token belongs to
    pub user_id: String,
    /// Granted scope
    pub scope: String,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    /// When the token was last used
    pub last_used_at: Option<DateTime<Utc>>,
    /// When the token was revoked (None if not revoked)
    pub revoked_at: Option<DateTime<Utc>>,
}

/// User information for token generation
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// User identifier
    pub id: String,
    /// Username
    pub username: String,
}

/// OAuth 2.0 Authorization Server Metadata (RFC 8414)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationServerMetadata {
    /// The authorization server's issuer identifier
    pub issuer: String,
    /// URL of the authorization endpoint
    pub authorization_endpoint: String,
    /// URL of the token endpoint
    pub token_endpoint: String,
    /// Supported response types
    pub response_types_supported: Vec<String>,
    /// Supported grant types
    pub grant_types_supported: Vec<String>,
    /// Supported token endpoint authentication methods
    pub token_endpoint_auth_methods_supported: Vec<String>,
    /// URL of the service documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_documentation: Option<String>,
    /// Supported UI locales
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_locales_supported: Option<Vec<String>>,
    /// PKCE code challenge methods supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,
    /// Supported scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    /// URL of the revocation endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
}

impl Default for AuthorizationServerMetadata {
    fn default() -> Self {
        Self {
            issuer: String::new(),
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_post".to_string(),
                "client_secret_basic".to_string(),
            ],
            service_documentation: None,
            ui_locales_supported: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string(), "plain".to_string()]),
            scopes_supported: None,
            revocation_endpoint: None,
        }
    }
}

/// OAuth 2.0 Protected Resource Metadata (RFC 8707)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedResourceMetadata {
    /// The protected resource identifier
    pub resource: String,
    /// Authorization servers that protect this resource
    pub authorization_servers: Vec<String>,
    /// Supported methods for presenting bearer tokens
    pub bearer_methods_supported: Vec<String>,
    /// URL of the resource documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_documentation: Option<String>,
    /// Supported signing algorithms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_signing_alg_values_supported: Option<Vec<String>>,
    /// Scopes required to access the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
}

impl Default for ProtectedResourceMetadata {
    fn default() -> Self {
        Self {
            resource: String::new(),
            authorization_servers: Vec::new(),
            bearer_methods_supported: vec!["header".to_string()],
            resource_documentation: None,
            resource_signing_alg_values_supported: None,
            scopes_supported: None,
        }
    }
}

/// Client credentials for OAuth client validation
#[derive(Debug, Clone)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Configuration for the OAuth server
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Authorization code expiry in seconds (default: 600 = 10 minutes)
    pub code_expiry_seconds: i64,
    /// Access token expiry in seconds (default: 86400 = 24 hours)
    pub token_expiry_seconds: i64,
    /// JWT secret for signing tokens (if JWT is used)
    pub jwt_secret: Option<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            code_expiry_seconds: 600,
            token_expiry_seconds: 86400,
            jwt_secret: None,
        }
    }
}
