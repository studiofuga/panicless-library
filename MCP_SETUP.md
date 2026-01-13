# Panicless Library MCP Server - Claude Desktop Integration

This guide explains how to connect Claude Desktop to the Panicless Library MCP server.

## Prerequisites

- Panicless Library running with Docker Compose
- Claude Desktop installed
- MCP server accessible at a public URL (e.g., `https://panicless.happycactus.org:8001`)

## Server Endpoints

The MCP server exposes the following endpoints:

- **Health Check**: `GET /health` (no auth required)
- **OpenAPI Schema**: `GET /openapi.json` (no auth required)
- **MCP Protocol (GET)**: `GET /mcp` (SSE stream, JWT auth required)
- **MCP Protocol (POST)**: `POST /mcp` (JSON-RPC over SSE, JWT auth required)

## Step 1: Get Your Access Token

First, you need to obtain a JWT access token from the backend API:

```bash
curl -X POST https://panicless.happycactus.org:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'
```

The response will include an `accessToken`. Save this token.

## Step 2: Verify MCP Server is Accessible

Test that your MCP server is working:

```bash
# Health check (no auth)
curl https://panicless.happycactus.org:8001/health

# OpenAPI schema (no auth)
curl https://panicless.happycactus.org:8001/openapi.json

# MCP endpoint test (requires JWT)
curl -X POST https://panicless.happycactus.org:8001/mcp \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocol_version": "2024-11-05",
      "capabilities": {},
      "client_info": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

## Step 3: Configure Claude Desktop

Edit your Claude Desktop configuration file:

**macOS/Linux**: `~/.claude/mcp-servers.json`
**Windows**: `%APPDATA%\Claude\mcp-servers.json`

Add the following configuration for your MCP server:

```json
{
  "mcpServers": {
    "panicless": {
      "url": "https://panicless.happycactus.org:8001/openapi.json",
      "auth": {
        "type": "bearer",
        "token": "YOUR_JWT_ACCESS_TOKEN"
      },
      "protocol": "sse"
    }
  }
}
```

Replace:
- `https://panicless.happycactus.org:8001` with your actual MCP server URL
- `YOUR_JWT_ACCESS_TOKEN` with the access token obtained in Step 1

## Step 4: Restart Claude Desktop

Close and reopen Claude Desktop for the configuration to take effect.

## Step 5: Test the Connection

In Claude Desktop, try using one of the available MCP tools:

- `search_books` - Search for books in your library
- `get_book_details` - Get details about a specific book
- `list_readings` - List your reading activities
- `get_reading_statistics` - Get reading statistics
- `create_book` - Add a new book
- `create_reading` - Create a new reading session

Example prompt in Claude Desktop:
> "Search my library for books about Rust programming and tell me what you find."

## Available Tools

All tools are user-scoped - they only return data for your authenticated user:

### Reading Tools
- **search_books**(query, author): Search books in your library
- **get_book_details**(book_id): Get full details of a book
- **list_readings**(limit, offset): Get your reading history
- **get_reading_statistics**(): Get your reading stats

### Management Tools
- **create_book**(title, author, isbn, published_year, description): Add a new book
- **create_reading**(book_id, start_date, end_date, rating, review): Start/finish reading
- **update_reading_review**(reading_id, rating, review): Update reading progress

### Analytics
- **find_similar_books**(book_id, limit): Find similar books to one you're reading

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
