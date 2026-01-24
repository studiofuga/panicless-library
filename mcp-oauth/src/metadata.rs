//! OAuth metadata endpoint handlers
//!
//! This module provides handlers for the OAuth 2.0 metadata endpoints
//! as defined in RFC 8414 (Authorization Server Metadata) and
//! RFC 8707 (Protected Resource Metadata).

use axum::{extract::State, Json};
use std::sync::Arc;

use crate::types::{AuthorizationServerMetadata, ProtectedResourceMetadata};

/// Trait for providing OAuth metadata
///
/// Implement this trait to customize the metadata returned by the
/// .well-known endpoints. The client is responsible for generating
/// the metadata based on their configuration.
pub trait MetadataProvider: Send + Sync {
    /// Get the authorization server metadata (RFC 8414)
    fn get_authorization_server_metadata(&self) -> AuthorizationServerMetadata;

    /// Get the protected resource metadata (RFC 8707)
    fn get_protected_resource_metadata(&self) -> ProtectedResourceMetadata;
}

/// Simple metadata provider with static configuration
///
/// This is a basic implementation that stores pre-configured metadata.
#[derive(Debug, Clone)]
pub struct StaticMetadataProvider {
    auth_server_metadata: AuthorizationServerMetadata,
    protected_resource_metadata: ProtectedResourceMetadata,
}

impl StaticMetadataProvider {
    /// Create a new static metadata provider
    pub fn new(
        auth_server_metadata: AuthorizationServerMetadata,
        protected_resource_metadata: ProtectedResourceMetadata,
    ) -> Self {
        Self {
            auth_server_metadata,
            protected_resource_metadata,
        }
    }

    /// Create a metadata provider from a base URL
    ///
    /// This is a convenience method that generates standard metadata
    /// based on a base URL.
    pub fn from_base_url(base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/');

        let auth_server_metadata = AuthorizationServerMetadata {
            issuer: base_url.to_string(),
            authorization_endpoint: format!("{}/authorize", base_url),
            token_endpoint: format!("{}/oauth/token", base_url),
            ..Default::default()
        };

        let protected_resource_metadata = ProtectedResourceMetadata {
            resource: format!("{}/api", base_url),
            authorization_servers: vec![base_url.to_string()],
            ..Default::default()
        };

        Self::new(auth_server_metadata, protected_resource_metadata)
    }
}

impl MetadataProvider for StaticMetadataProvider {
    fn get_authorization_server_metadata(&self) -> AuthorizationServerMetadata {
        self.auth_server_metadata.clone()
    }

    fn get_protected_resource_metadata(&self) -> ProtectedResourceMetadata {
        self.protected_resource_metadata.clone()
    }
}

/// Handler for /.well-known/oauth-authorization-server
///
/// Returns the OAuth 2.0 Authorization Server Metadata (RFC 8414).
pub async fn authorization_server_metadata<M: MetadataProvider>(
    State(provider): State<Arc<M>>,
) -> Json<AuthorizationServerMetadata> {
    Json(provider.get_authorization_server_metadata())
}

/// Handler for /.well-known/oauth-protected-resource
///
/// Returns the OAuth 2.0 Protected Resource Metadata (RFC 8707).
pub async fn protected_resource_metadata<M: MetadataProvider>(
    State(provider): State<Arc<M>>,
) -> Json<ProtectedResourceMetadata> {
    Json(provider.get_protected_resource_metadata())
}

/// Builder for creating metadata
#[derive(Debug, Clone, Default)]
pub struct MetadataBuilder {
    base_url: Option<String>,
    auth_metadata: AuthorizationServerMetadata,
    resource_metadata: ProtectedResourceMetadata,
}

impl MetadataBuilder {
    /// Create a new metadata builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL for the server
    pub fn base_url(mut self, url: &str) -> Self {
        let url = url.trim_end_matches('/').to_string();
        self.base_url = Some(url.clone());
        self.auth_metadata.issuer = url.clone();
        self.auth_metadata.authorization_endpoint = format!("{}/authorize", url);
        self.auth_metadata.token_endpoint = format!("{}/oauth/token", url);
        self.resource_metadata.resource = format!("{}/api", url);
        self.resource_metadata.authorization_servers = vec![url];
        self
    }

    /// Set custom authorization endpoint path
    pub fn authorization_endpoint(mut self, path: &str) -> Self {
        if let Some(base) = &self.base_url {
            self.auth_metadata.authorization_endpoint = format!("{}{}", base, path);
        }
        self
    }

    /// Set custom token endpoint path
    pub fn token_endpoint(mut self, path: &str) -> Self {
        if let Some(base) = &self.base_url {
            self.auth_metadata.token_endpoint = format!("{}{}", base, path);
        }
        self
    }

    /// Set service documentation URL
    pub fn service_documentation(mut self, url: &str) -> Self {
        self.auth_metadata.service_documentation = Some(url.to_string());
        self.resource_metadata.resource_documentation = Some(url.to_string());
        self
    }

    /// Set supported scopes
    pub fn scopes(mut self, scopes: Vec<String>) -> Self {
        self.auth_metadata.scopes_supported = Some(scopes.clone());
        self.resource_metadata.scopes_supported = Some(scopes);
        self
    }

    /// Set supported UI locales
    pub fn ui_locales(mut self, locales: Vec<String>) -> Self {
        self.auth_metadata.ui_locales_supported = Some(locales);
        self
    }

    /// Enable revocation endpoint
    pub fn revocation_endpoint(mut self, path: &str) -> Self {
        if let Some(base) = &self.base_url {
            self.auth_metadata.revocation_endpoint = Some(format!("{}{}", base, path));
        }
        self
    }

    /// Build the static metadata provider
    pub fn build(self) -> StaticMetadataProvider {
        StaticMetadataProvider::new(self.auth_metadata, self.resource_metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_builder() {
        let provider = MetadataBuilder::new()
            .base_url("https://example.com")
            .service_documentation("https://docs.example.com")
            .scopes(vec!["read".to_string(), "write".to_string()])
            .build();

        let auth_meta = provider.get_authorization_server_metadata();
        assert_eq!(auth_meta.issuer, "https://example.com");
        assert_eq!(auth_meta.authorization_endpoint, "https://example.com/authorize");
        assert_eq!(auth_meta.token_endpoint, "https://example.com/oauth/token");

        let resource_meta = provider.get_protected_resource_metadata();
        assert_eq!(resource_meta.resource, "https://example.com/api");
    }

    #[test]
    fn test_from_base_url() {
        let provider = StaticMetadataProvider::from_base_url("https://api.example.com/");

        let auth_meta = provider.get_authorization_server_metadata();
        assert_eq!(auth_meta.issuer, "https://api.example.com");
    }
}
