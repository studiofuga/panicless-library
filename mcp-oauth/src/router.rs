//! Router builder for OAuth endpoints
//!
//! This module provides utilities for building OAuth routes with Axum.

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{self, OAuthState},
    metadata::{self, MetadataProvider},
    storage::{AuthCodeStorage, ClientValidator, TokenStorage, UserProvider},
    types::OAuthConfig,
};

/// Create the public OAuth routes (token and metadata endpoints)
///
/// These routes do not require authentication.
///
/// # Type Parameters
///
/// * `S` - Storage type implementing all OAuth storage traits
/// * `M` - Metadata provider type
///
/// # Example
///
/// ```ignore
/// use mcp_oauth::router::create_oauth_routes;
///
/// let storage = Arc::new(MyStorage::new());
/// let metadata = Arc::new(MyMetadata::new());
/// let config = OAuthConfig::default();
///
/// let (token_routes, metadata_routes) = create_oauth_routes(storage, metadata, config);
/// ```
pub fn create_token_route<S>(storage: Arc<S>, config: OAuthConfig) -> Router
where
    S: ClientValidator + AuthCodeStorage + TokenStorage + UserProvider + Send + Sync + 'static,
{
    let oauth_state = OAuthState::new(storage, config);

    Router::new()
        .route("/oauth/token", post(handlers::token::<S>))
        .with_state(oauth_state)
}

/// Create the metadata routes
pub fn create_metadata_routes<M>(metadata: Arc<M>) -> Router
where
    M: MetadataProvider + 'static,
{
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(metadata::authorization_server_metadata::<M>),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(metadata::protected_resource_metadata::<M>),
        )
        .with_state(metadata)
}

/// Builder for OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthBuilder {
    config: OAuthConfig,
    token_path: String,
    auth_server_metadata_path: String,
    protected_resource_metadata_path: String,
}

impl Default for OAuthBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthBuilder {
    /// Create a new OAuth builder with default configuration
    pub fn new() -> Self {
        Self {
            config: OAuthConfig::default(),
            token_path: "/oauth/token".to_string(),
            auth_server_metadata_path: "/.well-known/oauth-authorization-server".to_string(),
            protected_resource_metadata_path: "/.well-known/oauth-protected-resource".to_string(),
        }
    }

    /// Set the authorization code expiry in seconds
    pub fn code_expiry_seconds(mut self, seconds: i64) -> Self {
        self.config.code_expiry_seconds = seconds;
        self
    }

    /// Set the access token expiry in seconds
    pub fn token_expiry_seconds(mut self, seconds: i64) -> Self {
        self.config.token_expiry_seconds = seconds;
        self
    }

    /// Set the JWT secret (if using JWT tokens)
    pub fn jwt_secret(mut self, secret: String) -> Self {
        self.config.jwt_secret = Some(secret);
        self
    }

    /// Set the path for the token endpoint
    pub fn token_path(mut self, path: &str) -> Self {
        self.token_path = path.to_string();
        self
    }

    /// Build the token route with custom path
    pub fn build_token_route<S>(self, storage: Arc<S>) -> (Router, OAuthState<S>)
    where
        S: ClientValidator + AuthCodeStorage + TokenStorage + UserProvider + Send + Sync + 'static,
    {
        let oauth_state = OAuthState::new(storage, self.config);
        let router = Router::new()
            .route(&self.token_path, post(handlers::token::<S>))
            .with_state(oauth_state.clone());

        (router, oauth_state)
    }

    /// Build the metadata routes with custom paths
    pub fn build_metadata_routes<M>(self, metadata: Arc<M>) -> Router
    where
        M: MetadataProvider + 'static,
    {
        Router::new()
            .route(
                &self.auth_server_metadata_path,
                get(metadata::authorization_server_metadata::<M>),
            )
            .route(
                &self.protected_resource_metadata_path,
                get(metadata::protected_resource_metadata::<M>),
            )
            .with_state(metadata)
    }

    /// Get the OAuth configuration
    pub fn get_config(&self) -> OAuthConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = OAuthBuilder::new();
        assert_eq!(builder.config.code_expiry_seconds, 600);
        assert_eq!(builder.config.token_expiry_seconds, 86400);
    }

    #[test]
    fn test_builder_config() {
        let builder = OAuthBuilder::new()
            .code_expiry_seconds(300)
            .token_expiry_seconds(3600);

        assert_eq!(builder.config.code_expiry_seconds, 300);
        assert_eq!(builder.config.token_expiry_seconds, 3600);
    }
}
