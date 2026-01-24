//! # mcp-oauth
//!
//! A reusable OAuth 2.0 library for MCP servers built with Axum.
//!
//! This crate provides OAuth 2.0 Authorization Code Grant flow implementation
//! (RFC 6749) with support for PKCE (RFC 7636) and OAuth metadata endpoints
//! (RFC 8414, RFC 8707).
//!
//! ## Features
//!
//! - **Authorization Code Grant Flow**: Complete implementation of OAuth 2.0
//!   authorization code flow
//! - **PKCE Support**: RFC 7636 Proof Key for Code Exchange for public clients
//! - **Metadata Endpoints**: RFC 8414 Authorization Server Metadata and
//!   RFC 8707 Protected Resource Metadata
//! - **Pluggable Storage**: Bring your own storage backend via traits
//! - **Axum Integration**: Built for Axum web framework with Tower middleware
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use mcp_oauth::{
//!     metadata::{MetadataBuilder, StaticMetadataProvider},
//!     router::OAuthRouterBuilder,
//!     storage::{AuthCodeStorage, ClientValidator, TokenStorage, UserProvider},
//!     types::OAuthConfig,
//! };
//! use std::sync::Arc;
//!
//! // 1. Implement the storage traits for your backend
//! struct MyStorage { /* your database connection */ }
//!
//! // 2. Create metadata provider
//! let metadata = MetadataBuilder::new()
//!     .base_url("https://api.example.com")
//!     .build();
//!
//! // 3. Build the OAuth router
//! let storage = Arc::new(MyStorage::new());
//! let (oauth_router, oauth_state) = OAuthRouterBuilder::new(storage, Arc::new(metadata))
//!     .token_expiry_seconds(3600)
//!     .build_public_routes();
//!
//! // 4. Add to your Axum app
//! let app = Router::new()
//!     .merge(oauth_router);
//! ```
//!
//! ## Storage Traits
//!
//! You must implement the following traits for your storage backend:
//!
//! - [`ClientValidator`]: Validate OAuth client credentials
//! - [`AuthCodeStorage`]: Store and retrieve authorization codes
//! - [`TokenStorage`]: Store and retrieve access tokens
//! - [`UserProvider`]: Lookup user information
//!
//! ## Endpoints
//!
//! The library provides the following endpoints:
//!
//! | Endpoint | Method | Description |
//! |----------|--------|-------------|
//! | `/oauth/authorize` | POST | Generate authorization code (requires auth) |
//! | `/oauth/token` | POST | Exchange code for token |
//! | `/.well-known/oauth-authorization-server` | GET | Server metadata |
//! | `/.well-known/oauth-protected-resource` | GET | Resource metadata |

pub mod error;
pub mod handlers;
pub mod metadata;
pub mod router;
pub mod storage;
pub mod types;

// Re-export commonly used types
pub use error::{OAuthError, OAuthResult};
pub use handlers::{authorize, token, AuthorizeContext, OAuthState};
pub use metadata::{MetadataBuilder, MetadataProvider, StaticMetadataProvider};
pub use router::{create_metadata_routes, create_token_route, OAuthBuilder};
pub use storage::{AuthCodeStorage, ClientValidator, OAuthStorage, TokenStorage, UserProvider};
pub use types::{
    AuthorizationServerMetadata, AuthorizeRequest, AuthorizeResponse, ClientCredentials,
    OAuthConfig, ProtectedResourceMetadata, StoredAuthCode, StoredToken, TokenRequest,
    TokenResponse, UserInfo,
};
