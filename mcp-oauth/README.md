# mcp-oauth

A reusable OAuth 2.0 library for MCP servers built with Axum.

## Features

- **Authorization Code Grant Flow**: Complete implementation of OAuth 2.0 authorization code flow (RFC 6749)
- **PKCE Support**: RFC 7636 Proof Key for Code Exchange for public clients
- **Metadata Endpoints**: RFC 8414 Authorization Server Metadata and RFC 8707 Protected Resource Metadata
- **Pluggable Storage**: Bring your own storage backend via traits
- **Axum Integration**: Built for Axum web framework with Tower middleware

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mcp-oauth = { path = "../mcp-oauth" }
```

## Quick Start

### 1. Implement Storage Traits

You need to implement four traits for your storage backend:

```rust
use async_trait::async_trait;
use mcp_oauth::{
    storage::{AuthCodeStorage, ClientValidator, TokenStorage, UserProvider},
    types::{ClientCredentials, StoredAuthCode, StoredToken, UserInfo},
    OAuthResult,
};

struct MyStorage { /* your database connection */ }

#[async_trait]
impl ClientValidator for MyStorage {
    async fn validate_client_id(&self, client_id: &str) -> OAuthResult<bool> {
        // Check if client_id exists in your database
    }

    async fn validate_client_credentials(&self, credentials: &ClientCredentials) -> OAuthResult<bool> {
        // Validate client_id and client_secret
    }

    async fn validate_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> OAuthResult<bool> {
        // Check if redirect_uri is allowed for this client
    }
}

#[async_trait]
impl AuthCodeStorage for MyStorage {
    async fn store_auth_code(&self, code: StoredAuthCode) -> OAuthResult<()> {
        // Store authorization code
    }

    async fn get_auth_code(&self, code: &str) -> OAuthResult<Option<StoredAuthCode>> {
        // Retrieve authorization code
    }

    async fn mark_code_used(&self, code: &str) -> OAuthResult<()> {
        // Mark code as used
    }
}

#[async_trait]
impl TokenStorage for MyStorage {
    async fn store_token(&self, token: StoredToken) -> OAuthResult<()> {
        // Store access token
    }

    async fn get_token(&self, token: &str) -> OAuthResult<Option<StoredToken>> {
        // Retrieve token
    }

    async fn revoke_token(&self, token: &str) -> OAuthResult<()> {
        // Revoke token
    }

    async fn touch_token(&self, token: &str) -> OAuthResult<()> {
        // Update last_used_at
    }
}

#[async_trait]
impl UserProvider for MyStorage {
    async fn get_user(&self, user_id: &str) -> OAuthResult<Option<UserInfo>> {
        // Get user information
    }
}
```

### 2. Configure Metadata

```rust
use mcp_oauth::metadata::MetadataBuilder;

let metadata = MetadataBuilder::new()
    .base_url("https://api.example.com")
    .service_documentation("https://docs.example.com")
    .scopes(vec!["read".to_string(), "write".to_string()])
    .build();
```

### 3. Build Routes

```rust
use mcp_oauth::router::{create_token_route, create_metadata_routes};
use mcp_oauth::types::OAuthConfig;
use std::sync::Arc;

let storage = Arc::new(MyStorage::new());
let metadata = Arc::new(metadata);

let config = OAuthConfig {
    code_expiry_seconds: 600,    // 10 minutes
    token_expiry_seconds: 86400, // 24 hours
    jwt_secret: None,
};

// Create routes
let token_routes = create_token_route(Arc::clone(&storage), config);
let metadata_routes = create_metadata_routes(metadata);

// Merge into your app
let app = Router::new()
    .merge(token_routes)
    .merge(metadata_routes);
```

### 4. Add Authorize Endpoint

The authorize endpoint requires authentication. You need to wrap it with your auth middleware:

```rust
use mcp_oauth::handlers::{authorize, AuthorizeContext, OAuthState};

async fn my_authorize_handler(
    State(oauth_state): State<OAuthState<MyStorage>>,
    my_auth: MyAuthExtractor,  // Your auth extractor
    Query(params): Query<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, OAuthError> {
    let context = AuthorizeContext {
        user_id: my_auth.user_id.to_string(),
    };

    authorize(State(oauth_state), context, Query(params)).await
}
```

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/oauth/authorize` | POST | Generate authorization code (requires auth) |
| `/oauth/token` | POST | Exchange code for token |
| `/.well-known/oauth-authorization-server` | GET | Server metadata (RFC 8414) |
| `/.well-known/oauth-protected-resource` | GET | Resource metadata (RFC 8707) |

## OAuth Flow

1. **Authorization Request**: Client redirects user to `/oauth/authorize` with:
   - `client_id`: The client identifier
   - `redirect_uri`: Where to redirect after authorization
   - `response_type`: Must be "code"
   - `scope`: Requested scopes (optional)
   - `state`: CSRF protection token (optional)
   - `code_challenge`: PKCE challenge (optional)
   - `code_challenge_method`: "S256" or "plain" (optional)

2. **Authorization Response**: Server returns:
   - `code`: The authorization code
   - `state`: The state parameter (if provided)

3. **Token Request**: Client exchanges code at `/oauth/token` with:
   - `client_id`: The client identifier
   - `client_secret`: The client secret (in body or Basic auth header)
   - `code`: The authorization code
   - `grant_type`: Must be "authorization_code"
   - `redirect_uri`: Must match the original request
   - `code_verifier`: PKCE verifier (if challenge was provided)

4. **Token Response**: Server returns:
   - `access_token`: The access token
   - `token_type`: "Bearer"
   - `expires_in`: Token lifetime in seconds
   - `scope`: Granted scopes

## Example

See `examples/basic.rs` for a complete working example with in-memory storage.

```bash
cargo run --example basic
```

## License

MIT
