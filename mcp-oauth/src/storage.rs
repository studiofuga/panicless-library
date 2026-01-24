//! Storage traits for OAuth data persistence
//!
//! This module defines the traits that clients must implement to provide
//! their own storage backend for authorization codes and access tokens.

use async_trait::async_trait;

use crate::{
    error::OAuthResult,
    types::{ClientCredentials, StoredAuthCode, StoredToken, UserInfo},
};

/// Trait for validating OAuth clients
///
/// Implement this trait to define how client credentials are validated.
#[async_trait]
pub trait ClientValidator: Send + Sync {
    /// Validate client_id and return true if it's a known client
    async fn validate_client_id(&self, client_id: &str) -> OAuthResult<bool>;

    /// Validate client credentials (client_id and client_secret)
    async fn validate_client_credentials(&self, credentials: &ClientCredentials) -> OAuthResult<bool>;

    /// Validate redirect URI for a client
    async fn validate_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> OAuthResult<bool>;
}

/// Trait for storing and retrieving authorization codes
///
/// Implement this trait to provide persistent storage for authorization codes.
#[async_trait]
pub trait AuthCodeStorage: Send + Sync {
    /// Store a new authorization code
    async fn store_auth_code(&self, code: StoredAuthCode) -> OAuthResult<()>;

    /// Retrieve an authorization code by its value
    async fn get_auth_code(&self, code: &str) -> OAuthResult<Option<StoredAuthCode>>;

    /// Mark an authorization code as used
    async fn mark_code_used(&self, code: &str) -> OAuthResult<()>;

    /// Delete expired authorization codes (optional cleanup)
    async fn cleanup_expired_codes(&self) -> OAuthResult<u64> {
        Ok(0)
    }
}

/// Trait for storing and retrieving access tokens
///
/// Implement this trait to provide persistent storage for access tokens.
#[async_trait]
pub trait TokenStorage: Send + Sync {
    /// Store a new access token
    async fn store_token(&self, token: StoredToken) -> OAuthResult<()>;

    /// Retrieve a token by its value
    async fn get_token(&self, token: &str) -> OAuthResult<Option<StoredToken>>;

    /// Revoke a token
    async fn revoke_token(&self, token: &str) -> OAuthResult<()>;

    /// Update the last_used_at timestamp for a token
    async fn touch_token(&self, token: &str) -> OAuthResult<()>;

    /// Delete expired tokens (optional cleanup)
    async fn cleanup_expired_tokens(&self) -> OAuthResult<u64> {
        Ok(0)
    }
}

/// Trait for retrieving user information
///
/// Implement this trait to provide user lookup functionality.
#[async_trait]
pub trait UserProvider: Send + Sync {
    /// Get user information by user ID
    async fn get_user(&self, user_id: &str) -> OAuthResult<Option<UserInfo>>;
}

/// Combined storage trait for all OAuth storage needs
///
/// This is a convenience trait that combines all storage traits.
/// You can implement this trait or implement the individual traits separately.
pub trait OAuthStorage: ClientValidator + AuthCodeStorage + TokenStorage + UserProvider {}

/// Blanket implementation for any type that implements all storage traits
impl<T> OAuthStorage for T where T: ClientValidator + AuthCodeStorage + TokenStorage + UserProvider {}
