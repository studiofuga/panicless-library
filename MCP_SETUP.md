# Panicless Library MCP Integration - Claude Desktop Setup

This guide explains how to integrate Panicless Library with Claude Desktop using two deployment options:
1. **Backend MCP** (HTTP/SSE) - For remote access with OAuth2
2. **Standalone MCP Server** (stdio) - For local Claude Desktop usage

## Prerequisites

- Panicless Library running with Docker Compose
- Claude Desktop installed
- For remote setup: MCP backend/server accessible at a public URL

## Deployment Options

### Option 1: Backend MCP (HTTP/SSE) - Recommended for Remote Access

The backend includes MCP endpoints on the same host as the REST API. This solves OAuth2 integration issues and is ideal for remote access.

**Endpoint**: `http://localhost:8080/mcp` (in Docker) or your backend URL
**Authentication**: JWT Bearer token
**Protocol**: HTTP Server-Sent Events (SSE)
**Use Case**: Remote Claude Desktop, web applications, OAuth2 integration

#### Advantages
- ✅ Same host as REST API (solves CORS/OAuth2 issues)
- ✅ Shared authentication context (same JWT_SECRET)
- ✅ No additional service to manage
- ✅ Single source of truth for database queries

### Option 2: Standalone MCP Server (stdio) - For Local Development

The standalone mcp-server provides stdio-based MCP for direct Claude Desktop integration. Useful for local development and testing.

**Endpoint**: stdio (direct connection to Claude Desktop)
**Port**: 8081 (for health checks)
**Authentication**: Configured at startup (environment-based)
**Protocol**: stdio or HTTP/SSE
**Use Case**: Local Claude Desktop integration, development

#### Advantages
- ✅ Direct connection to Claude Desktop (no OAuth2 needed)
- ✅ Lightweight, focused tool
- ✅ Good for local development and testing

## Server Endpoints

### Backend MCP Endpoints (Port 8080)

- **Health Check**: `GET /health` (no auth required)
- **OpenAPI Schema**: `GET /openapi.json` (no auth required)
- **MCP Protocol (GET)**: `GET /mcp` (SSE stream, JWT auth required)
- **MCP Protocol (POST)**: `POST /mcp` (JSON-RPC over SSE, JWT auth required)

### Standalone MCP Server Endpoints (Port 8081)

- **Health Check**: `GET /health` (no auth required)
- **OpenAPI Schema**: `GET /openapi.json` (no auth required)
- **MCP Protocol (POST)**: `POST /mcp` (JSON-RPC over SSE, JWT auth required)

## Setup Instructions

### For Option 1: Backend MCP (HTTP/SSE) - Remote Access

#### Step 1: Get Your Access Token

Obtain a JWT access token from the backend API:

```bash
# Replace with your actual backend URL
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'
```

The response will include an `accessToken`. Save this token.

Example response:
```json
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": 1,
    "username": "your_username"
  }
}
```

#### Step 2: Verify Backend MCP is Accessible

Test that the backend MCP endpoint is working:

```bash
# Health check (no auth)
curl http://localhost:8080/health

# OpenAPI schema (no auth)
curl http://localhost:8080/openapi.json

# MCP endpoint test (requires JWT)
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    },
    "serverInfo": {
      "name": "panicless-backend-mcp",
      "version": "0.1.0"
    }
  },
  "id": 1
}
```

#### Step 3: Configure Claude Desktop

Edit your Claude Desktop configuration file:

**macOS/Linux**: `~/.claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Add the following configuration for the backend MCP:

```json
{
  "mcpServers": {
    "panicless-backend": {
      "url": "http://localhost:8080/mcp",
      "headers": {
        "Authorization": "Bearer YOUR_ACCESS_TOKEN"
      }
    }
  }
}
```

For remote/production setup, replace:
- `http://localhost:8080` with your actual backend URL (e.g., `https://panicless.example.com`)
- `YOUR_ACCESS_TOKEN` with the JWT token obtained in Step 1

#### Step 4: Restart Claude Desktop

Close and reopen Claude Desktop for the configuration to take effect.

#### Step 5: Test the Connection

In Claude Desktop, try one of the available tools:
> "Search my library for books about Rust programming"

---

### For Option 2: Standalone MCP Server (stdio) - Local Development

For local development, you can use the standalone mcp-server with stdio connection.

#### Step 1: Build and Run Standalone MCP Server

```bash
cd panicless-library
cd mcp-server

# Build the mcp-server
cargo build --release

# Run with environment variables pointing to local database
DATABASE_URL="postgresql://postgres:postgres@localhost:5432/panicless" \
./target/release/panicless-mcp-server
```

#### Step 2: Configure Claude Desktop for stdio

Edit your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "panicless-local": {
      "command": "/path/to/panicless-library/mcp-server/target/release/panicless-mcp-server",
      "args": [],
      "env": {
        "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/panicless",
        "JWT_SECRET": "your-jwt-secret"
      }
    }
  }
}
```

#### Step 3: Restart Claude Desktop

Close and reopen Claude Desktop to test the stdio connection.

## Available MCP Tools

All tools are user-scoped - they only return data for your authenticated user:

### Search & Discovery
- **search_books**(query, author, year, limit): Search for books in your library
- **get_book_details**(book_id): Get full details about a specific book
- **find_similar_books**(book_id): Find books by the same author

### Reading Management
- **list_readings**(status, year): List your reading sessions (filter by status: current/completed/all)
- **create_reading**(book_id, start_date, end_date): Record a new reading session
- **update_reading_review**(reading_id, rating, notes): Add or update a review and rating

### Library Management
- **create_book**(title, author, isbn, publication_year, publisher, pages, language, description): Add a new book to your library

### Analytics
- **get_reading_statistics**(): Get comprehensive reading statistics (books read, average rating, yearly breakdown)

## Troubleshooting

### Backend MCP Issues

**Issue**: "Connection refused" when connecting to backend MCP
- Verify the backend is running: `curl http://localhost:8080/health`
- Ensure JWT token is valid and not expired
- Check CORS settings in backend config

**Issue**: "Authentication failed"
- Verify the JWT token is correct and still valid
- Get a fresh token with `/api/auth/login`
- Check that the token is in the correct format in Claude Desktop config

**Issue**: Tools not listed in Claude Desktop
- Restart Claude Desktop after editing config
- Check the backend logs: `docker-compose logs backend`
- Verify MCP endpoint returns tools list: `curl http://localhost:8080/mcp`

### Standalone MCP Server Issues

**Issue**: "Command not found" when starting mcp-server
- Verify the path to the binary is correct
- Ensure the binary was built: `cargo build --release` in mcp-server directory
- Check file permissions: `chmod +x ./target/release/panicless-mcp-server`

**Issue**: Database connection errors
- Verify DATABASE_URL is correct
- Ensure PostgreSQL is running and accessible
- Check database migrations have run: `docker-compose logs postgres`

## OAuth2 Flow Verification

To verify the OAuth2 flow works end-to-end:

```bash
# 1. Get a JWT token from backend login
JWT=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"your_user","password":"your_pass"}' | jq -r '.accessToken')

# 2. Verify token works with MCP
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
  -H "Content-Type: application/json" \
  -d "{\"client_id\":\"panicless-library\",\"client_secret\":\"YOUR_OAUTH_CLIENT_SECRET\",\"code\":\"$CODE\",\"grant_type\":\"authorization_code\",\"redirect_uri\":\"http://localhost/callback\"}" \
  | jq -r '.jwt_token')

# 4. Test with MCP server
curl -s -X POST https://panicless.happycactus.org:8001/mcp \
  -H "Authorization: Bearer $OAUTH_JWT" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | jq .
```

If step 2 returns "InvalidSignature" error, your JWT_SECRET configuration is wrong. See the critical note above.

## Troubleshooting

### "Invalid content type, expected text/event-stream"

This error occurs when:
1. The MCP server URL is incorrect
2. The server is not running
3. HTTPS certificate is not valid

**Solution**:
- Verify the URL in your Claude Desktop config matches your server
- Check that Docker Compose services are running: `docker compose ps`
- Ensure your SSL certificate is valid

### "Unauthorized" or 401 errors

The JWT token has expired or is invalid.

**Solution**:
1. Get a fresh access token from the backend
2. Update the token in `mcp-servers.json`
3. Restart Claude Desktop

### Tools are not appearing

The server may not be responding to tool list requests.

**Solution**:
1. Check server logs: `docker compose logs mcp-server`
2. Verify JWT token is valid
3. Try the `/openapi.json` endpoint directly with your token

### Connection test fails in MCP Inspector

Make sure you're using the correct URL format and authorization header.

```bash
# Correct format:
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Token Refresh

JWT access tokens expire after 1 hour by default. Before the token expires:

```bash
curl -X POST https://panicless.happycactus.org:8000/api/auth/refresh \
  -H "Authorization: Bearer YOUR_REFRESH_TOKEN"
```

Then update the token in your Claude Desktop config and restart.

## Security Notes

- **Never share your access tokens** with anyone
- Tokens contain your user ID and can access your personal library
- Always use HTTPS for remote connections
- Regularly rotate tokens in production
- Consider using API keys or service accounts instead of personal tokens

## More Information

For more details on MCP and remote connectors, see:
- [Anthropic MCP Documentation](https://modelcontextprotocol.io/)
- [Claude Desktop Remote Connector Guide](https://support.claude.com/en/articles/11503834-building-custom-connectors-via-remote-mcp-servers)
