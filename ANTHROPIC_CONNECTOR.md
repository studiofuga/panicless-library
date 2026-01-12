# Anthropic OAuth2 Connector Setup

This guide explains how to connect your Panicless Library backend to Claude (via Anthropic's web interface) using OAuth2 authentication, so that Claude can securely access your personal library data as tools.

## Overview

Once configured, you can:
- Ask Claude to summarize your reading statistics
- Query your library for books by author or topic
- Get reading recommendations based on your completed books
- Extract insights from your reading history
- All without sharing your data with Anthropic - Claude accesses it through your own backend using OAuth2

## Prerequisites

1. **Panicless Library Backend** running and accessible from the internet
2. **HTTPS enabled** (required by Anthropic for OAuth2)
3. **Anthropic account** with access to Claude
4. **User account** in Panicless Library (registered and logged in)

## Architecture

The OAuth2 flow works as follows:

```
┌─────────────────────────────────────────────────────────────┐
│ User on claude.ai clicks "Connect Panicless Library"         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Redirects to Backend: /oauth/authorize                      │
│ (User must be logged in to Panicless)                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Backend generates authorization code (expires in 10 min)    │
│ Redirects back to Claude with code                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Claude (backend) exchanges code for access token            │
│ POST /oauth/token with client_id, client_secret, code       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Claude receives:                                            │
│ - access_token (for tracking/revocation)                    │
│ - jwt_token (for accessing protected endpoints)             │
│ - expires_in: 24 hours                                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Claude uses jwt_token for all API requests                  │
│ Authorization: Bearer <jwt_token>                           │
└─────────────────────────────────────────────────────────────┘
```

## Step 1: Get Your OAuth2 Credentials

Your backend generates OAuth2 credentials automatically. They are:

- **Client ID**: Set via `OAUTH_CLIENT_ID` environment variable
  - Default (dev): `panicless-library-dev`
  - Production: Use a unique, random value

- **Client Secret**: Set via `OAUTH_CLIENT_SECRET` environment variable
  - Default (dev): `dev-secret-change-in-production`
  - Production: Use a strong, random secret (min 32 characters)

In `docker-compose.yml` or `.env`:

```bash
OAUTH_CLIENT_ID=your-unique-client-id-here
OAUTH_CLIENT_SECRET=your-strong-secret-here-min-32-chars
```

## Step 2: Verify OpenAPI Schema

The backend exposes a complete OpenAPI specification at:

```
https://your-domain.com/openapi.json
```

or locally (for testing):

```
http://localhost:8080/openapi.json
```

Verify it's accessible by visiting this URL in your browser. You should see a JSON document describing all available endpoints.

## Step 3: Set Up HTTPS (Production Required)

**Important**: Anthropic only accepts HTTPS URLs for OAuth2 connectors.

For production deployment:

1. Use a reverse proxy (Nginx, Apache, Caddy)
2. Configure SSL/TLS certificates (e.g., Let's Encrypt)
3. Forward requests to your backend on `http://localhost:8080`

Example Nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Step 4: Connect to Claude (Web Interface)

1. Log in to https://claude.ai

2. In the conversation, look for settings/tools menu (exact location may vary)

3. Select "Add Custom Tool" or "Connect Integration"

4. Choose "Use OpenAPI Spec" or "REST API"

5. Fill in the form:

   - **Name**: `Panicless Library`
   - **Schema URL**: `https://your-domain.com/openapi.json`
   - **Authentication Type**: `OAuth2`
   - **Auth Type**: `Authorization Code`
   - **Authorization URL**: `https://your-domain.com/oauth/authorize`
   - **Token URL**: `https://your-domain.com/oauth/token`
   - **Client ID**: Your OAUTH_CLIENT_ID value
   - **Client Secret**: Your OAUTH_CLIENT_SECRET value
   - **Scope**: `all` (or custom scopes)

6. Click "Authorize" or "Connect"

7. You'll be redirected to your Panicless Library backend to log in (if not already logged in)

8. After successful login, Claude receives the tokens and is ready to use your library

## Step 5: Test the Connection

Try asking Claude questions like:

```
"How many books have I read?"
"Show me my reading statistics"
"What's the longest book I've read?"
"List all books by Stephen King"
```

Claude should:
1. Use the `/api/readings/stats` tool to fetch statistics
2. Use the `/api/books` tool to fetch book data
3. Parse the data and provide a natural response

## OAuth2 Flow Details

### Authorization Code Generation

**Endpoint**: `POST /oauth/authorize`

Requires: User authentication (JWT token in session)

Request:
```json
{
  "client_id": "your-client-id",
  "redirect_uri": "https://api.anthropic.com/oauth/callback",
  "response_type": "code",
  "scope": "all",
  "state": "random-state-string"
}
```

Response:
```json
{
  "code": "generated-auth-code",
  "state": "random-state-string"
}
```

The code expires in **10 minutes**.

### Token Exchange

**Endpoint**: `POST /oauth/token`

Does NOT require authentication (uses client credentials)

Request:
```json
{
  "client_id": "your-client-id",
  "client_secret": "your-client-secret",
  "code": "generated-auth-code",
  "grant_type": "authorization_code",
  "redirect_uri": "https://api.anthropic.com/oauth/callback"
}
```

Response:
```json
{
  "access_token": "opaque-token-for-tracking",
  "token_type": "Bearer",
  "expires_in": 86400,
  "scope": "all",
  "jwt_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

Claude uses the `jwt_token` to access protected endpoints.

## Security Considerations

1. **Token Expiry**: Tokens expire after 24 hours. Claude will need to re-authenticate.

2. **Token Revocation**: You can revoke tokens by:
   - Deleting from `oauth_tokens` table
   - Disconnecting from Claude settings
   - Rotating `OAUTH_CLIENT_SECRET`

3. **Authorization Code**: Single-use only, expires in 10 minutes

4. **HTTPS Required**: All OAuth2 flow must be over HTTPS

5. **Client Secret**: Treat like a password, never commit to version control

6. **Data Privacy**: Your library data accessed by Claude through your backend - never stored on Anthropic's servers

7. **Scope Validation**: Current implementation supports `all` scope (full access)

## Available API Endpoints

Once connected, Claude can use these endpoints:

### Books
- `GET /api/books` - List all books
- `GET /api/books/{id}` - Get specific book
- `POST /api/books` - Create new book

### Readings
- `GET /api/readings` - List reading sessions
- `GET /api/readings/{id}` - Get reading session
- `GET /api/readings/stats` - Get statistics

### Connectors
- `GET /api/connectors` - List AI provider connectors

## Troubleshooting

### "Connection Failed" or "Invalid Schema URL"
- Verify HTTPS is working: `curl -v https://your-domain.com/openapi.json`
- Check that OpenAPI schema is publicly accessible
- Verify backend is running: `curl https://your-domain.com/health`

### "Invalid Client ID" or "Invalid Client Secret"
- Verify credentials in your oauth configuration
- Check environment variables are set correctly
- Ensure no extra whitespace in credentials

### "Authorization Code Not Found"
- Code has likely expired (10 minute limit)
- User not logged in when authorizing
- Redirect URI mismatch

### "Token Expired"
- Tokens last 24 hours
- Need to re-authorize Claude
- Old tokens automatically become invalid

### Claude Can't Access Data
- Check backend logs for errors
- Verify JWT token is in Authorization header
- Ensure user has permission to access the resource
- Confirm backend is running and database is reachable

### HTTPS Certificate Issues
- For Let's Encrypt: `certbot certonly -d your-domain.com`
- For self-signed: Use a proper CA-signed certificate in production
- Check certificate validity: `openssl s_client -connect your-domain.com:443`

## Environment Variables

Add to your `.env` file or docker-compose environment:

```bash
# OAuth2 Configuration
OAUTH_CLIENT_ID=your-unique-client-id
OAUTH_CLIENT_SECRET=your-strong-secret

# JWT Configuration
JWT_SECRET=your-jwt-secret-key
JWT_ACCESS_TOKEN_EXPIRY=3600

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Database
DATABASE_URL=postgresql://user:password@host:5432/panicless

# Encryption
ENCRYPTION_KEY=your-encryption-key-base64-encoded
```

## Revoking Access

To revoke Claude's access to your library:

1. Log in to claude.ai

2. Go to settings → Connected Tools/Integrations

3. Find "Panicless Library" and click "Disconnect"

4. (Optional) Delete all tokens from database:
   ```sql
   DELETE FROM oauth_tokens WHERE client_id = 'anthropic';
   ```

5. (Optional) Rotate `OAUTH_CLIENT_SECRET` in your environment

Your library data remains secure - Claude will no longer have access.

## Advanced: Custom Scopes

To implement custom authorization scopes:

1. Modify `POST /oauth/authorize` to validate scope parameter
2. Store requested scope in `oauth_codes` table
3. Add scope validation in protected endpoints
4. Include scope in JWT claims for enforcement

Example:
```rust
if let Some(scope) = requested_scope {
    if scope == "read-only" {
        // Only allow GET requests
    }
}
```

## Support

For issues or questions:
- Check backend logs: `docker-compose logs -f backend`
- Verify environment variables: `echo $OAUTH_CLIENT_ID`
- Test endpoints manually: `curl -v https://your-domain.com/openapi.json`
- Review this guide for troubleshooting section
