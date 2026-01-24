//! Basic example of using mcp-oauth with in-memory storage
//!
//! This example demonstrates how to:
//! 1. Implement the storage traits with an in-memory backend
//! 2. Configure metadata
//! 3. Build OAuth routes
//! 4. Integrate with an Axum application
//!
//! Run with: cargo run --example basic

use async_trait::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use mcp_oauth::{
    error::{OAuthError, OAuthResult},
    handlers::{self, AuthorizeContext, OAuthState},
    metadata::MetadataBuilder,
    router::{create_metadata_routes, create_token_route},
    storage::{AuthCodeStorage, ClientValidator, TokenStorage, UserProvider},
    types::{
        AuthorizeRequest, AuthorizeResponse, ClientCredentials, OAuthConfig, StoredAuthCode,
        StoredToken, UserInfo,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ============================================================================
// In-Memory Storage Implementation
// ============================================================================

/// In-memory storage for demonstration purposes
///
/// In a real application, you would use a database like PostgreSQL, Redis, etc.
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    /// Registered OAuth clients
    clients: RwLock<HashMap<String, String>>, // client_id -> client_secret
    /// Allowed redirect URIs per client
    redirect_uris: RwLock<HashMap<String, Vec<String>>>, // client_id -> [redirect_uris]
    /// Stored authorization codes
    auth_codes: RwLock<HashMap<String, StoredAuthCode>>,
    /// Stored access tokens
    tokens: RwLock<HashMap<String, StoredToken>>,
    /// User database
    users: RwLock<HashMap<String, UserInfo>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        let storage = Self::default();

        // Register a test client
        {
            let mut clients = storage.clients.write().unwrap();
            clients.insert("test-client".to_string(), "test-secret".to_string());
        }

        // Register allowed redirect URIs
        {
            let mut uris = storage.redirect_uris.write().unwrap();
            uris.insert(
                "test-client".to_string(),
                vec![
                    "http://localhost:3000/callback".to_string(),
                    "http://127.0.0.1:3000/callback".to_string(),
                ],
            );
        }

        // Add a test user
        {
            let mut users = storage.users.write().unwrap();
            users.insert(
                "user-123".to_string(),
                UserInfo {
                    id: "user-123".to_string(),
                    username: "testuser".to_string(),
                },
            );
        }

        storage
    }
}

#[async_trait]
impl ClientValidator for InMemoryStorage {
    async fn validate_client_id(&self, client_id: &str) -> OAuthResult<bool> {
        let clients = self.clients.read().unwrap();
        Ok(clients.contains_key(client_id))
    }

    async fn validate_client_credentials(
        &self,
        credentials: &ClientCredentials,
    ) -> OAuthResult<bool> {
        let clients = self.clients.read().unwrap();
        match clients.get(&credentials.client_id) {
            Some(secret) => Ok(secret == &credentials.client_secret),
            None => Ok(false),
        }
    }

    async fn validate_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> OAuthResult<bool> {
        let uris = self.redirect_uris.read().unwrap();
        match uris.get(client_id) {
            Some(allowed) => Ok(allowed.iter().any(|u| u == redirect_uri)),
            None => Ok(false),
        }
    }
}

#[async_trait]
impl AuthCodeStorage for InMemoryStorage {
    async fn store_auth_code(&self, code: StoredAuthCode) -> OAuthResult<()> {
        let mut codes = self.auth_codes.write().unwrap();
        codes.insert(code.code.clone(), code);
        Ok(())
    }

    async fn get_auth_code(&self, code: &str) -> OAuthResult<Option<StoredAuthCode>> {
        let codes = self.auth_codes.read().unwrap();
        Ok(codes.get(code).cloned())
    }

    async fn mark_code_used(&self, code: &str) -> OAuthResult<()> {
        let mut codes = self.auth_codes.write().unwrap();
        if let Some(stored) = codes.get_mut(code) {
            stored.used_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn cleanup_expired_codes(&self) -> OAuthResult<u64> {
        let mut codes = self.auth_codes.write().unwrap();
        let now = Utc::now();
        let before = codes.len();
        codes.retain(|_, c| c.expires_at > now);
        Ok((before - codes.len()) as u64)
    }
}

#[async_trait]
impl TokenStorage for InMemoryStorage {
    async fn store_token(&self, token: StoredToken) -> OAuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.token.clone(), token);
        Ok(())
    }

    async fn get_token(&self, token: &str) -> OAuthResult<Option<StoredToken>> {
        let tokens = self.tokens.read().unwrap();
        Ok(tokens.get(token).cloned())
    }

    async fn revoke_token(&self, token: &str) -> OAuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        if let Some(stored) = tokens.get_mut(token) {
            stored.revoked_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn touch_token(&self, token: &str) -> OAuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        if let Some(stored) = tokens.get_mut(token) {
            stored.last_used_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> OAuthResult<u64> {
        let mut tokens = self.tokens.write().unwrap();
        let now = Utc::now();
        let before = tokens.len();
        tokens.retain(|_, t| t.expires_at > now && t.revoked_at.is_none());
        Ok((before - tokens.len()) as u64)
    }
}

#[async_trait]
impl UserProvider for InMemoryStorage {
    async fn get_user(&self, user_id: &str) -> OAuthResult<Option<UserInfo>> {
        let users = self.users.read().unwrap();
        Ok(users.get(user_id).cloned())
    }
}

// ============================================================================
// Application Routes
// ============================================================================

/// Simple login request
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    #[allow(dead_code)]
    password: String,
}

/// Simple login response with a session token
#[derive(Debug, Serialize)]
struct LoginResponse {
    session_token: String,
    user_id: String,
}

/// Fake login endpoint (in a real app, validate credentials)
async fn login(Json(_req): Json<LoginRequest>) -> impl IntoResponse {
    // In a real app, validate username/password against database
    // For demo, we just return a fake session
    let response = LoginResponse {
        session_token: "fake-session-token".to_string(),
        user_id: "user-123".to_string(),
    };
    (StatusCode::OK, Json(response))
}

/// Protected authorize endpoint
///
/// This wraps the OAuth authorize handler with session validation.
/// In a real app, you would validate the session token and get the user ID.
async fn authorize_handler(
    State(oauth_state): State<OAuthState<InMemoryStorage>>,
    Query(params): Query<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, OAuthError> {
    // In a real app, validate session from Authorization header or cookie
    // and get the actual user_id
    let context = AuthorizeContext {
        user_id: "user-123".to_string(),
    };

    handlers::authorize(State(oauth_state), context, Query(params)).await
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

// ============================================================================
// Main Application
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,mcp_oauth=debug".to_string()),
        ))
        .init();

    tracing::info!("Starting OAuth example server");

    // Create storage
    let storage = Arc::new(InMemoryStorage::new());

    // Create metadata provider
    let base_url = "http://localhost:3000";
    let metadata = Arc::new(
        MetadataBuilder::new()
            .base_url(base_url)
            .service_documentation("https://github.com/your/repo")
            .scopes(vec![
                "read".to_string(),
                "write".to_string(),
                "all".to_string(),
            ])
            .build(),
    );

    // Create OAuth configuration
    let oauth_config = OAuthConfig {
        code_expiry_seconds: 600,    // 10 minutes
        token_expiry_seconds: 86400, // 24 hours
        jwt_secret: None,            // Not using JWT in this example
    };

    // Create OAuth state for the authorize handler
    let oauth_state = OAuthState::new(Arc::clone(&storage), oauth_config.clone());

    // Build routes using the helper functions
    let token_routes = create_token_route(Arc::clone(&storage), oauth_config);
    let metadata_routes = create_metadata_routes(metadata);

    // Build the application
    // Note: We use .merge() to combine routers with different state types
    let app = Router::new()
        // Health check
        .route("/health", get(health))
        // Login endpoint (for demo)
        .route("/login", post(login))
        // OAuth authorize endpoint (protected, needs auth)
        .route("/oauth/authorize", post(authorize_handler))
        .with_state(oauth_state)
        // Merge OAuth token route
        .merge(token_routes)
        // Merge metadata routes
        .merge(metadata_routes);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);
    tracing::info!("");
    tracing::info!("Test endpoints:");
    tracing::info!("  GET  http://localhost:3000/health");
    tracing::info!(
        "  GET  http://localhost:3000/.well-known/oauth-authorization-server"
    );
    tracing::info!(
        "  GET  http://localhost:3000/.well-known/oauth-protected-resource"
    );
    tracing::info!("");
    tracing::info!("OAuth flow:");
    tracing::info!("  1. POST /oauth/authorize?client_id=test-client&redirect_uri=http://localhost:3000/callback&response_type=code");
    tracing::info!(
        "  2. POST /oauth/token with client_id, client_secret, code, grant_type, redirect_uri"
    );

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
