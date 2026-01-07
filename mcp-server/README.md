# Panicless MCP Server

Model Context Protocol (MCP) server for Panicless Library - provides read-only access to your book library via AI assistants like Claude and Gemini.

## What is MCP?

The [Model Context Protocol](https://modelcontextprotocol.io) is an open protocol that enables AI assistants to securely connect to local data sources and tools. This MCP server allows Claude, Gemini, or other MCP-compatible AI assistants to query your personal library database.

## Features

- **Read-Only Access**: Safe, read-only access to your library data
- **User-Scoped**: All queries are scoped to a specific user ID
- **5 Powerful Tools**:
  1. `search_books` - Search books by title, author, or year
  2. `get_book_details` - Get full book information with reading history
  3. `list_readings` - List reading records with filters
  4. `get_reading_statistics` - Get comprehensive reading stats
  5. `find_similar_books` - Find books by the same author

## Prerequisites

- Rust 1.85+ (edition 2021, supports edition 2024 dependencies)
- PostgreSQL database (from main project)
- MCP-compatible AI client (Claude Desktop, etc.)

## Installation

### 1. Build the MCP Server

```bash
cd mcp-server
cargo build --release
```

The binary will be at `target/release/panicless-mcp-server`.

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env and set your DATABASE_URL
```

### 3. Test the Server

```bash
cargo run
```

The server will start and wait for JSON-RPC messages on stdin. You can test it manually:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run
```

## Configuration with Claude Desktop

### Option 1: Using claude_desktop_config.json

Add to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "panicless-library": {
      "command": "/path/to/panicless-library/mcp-server/target/release/panicless-mcp-server",
      "env": {
        "DATABASE_URL": "postgres://panicless:panicless_dev@localhost:5432/panicless_library",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Option 2: Using a Shell Script

Create a wrapper script `run-mcp-server.sh`:

```bash
#!/bin/bash
cd /path/to/panicless-library/mcp-server
export DATABASE_URL="postgres://panicless:panicless_dev@localhost:5432/panicless_library"
export RUST_LOG="info"
exec ./target/release/panicless-mcp-server
```

Then reference it in Claude Desktop config:

```json
{
  "mcpServers": {
    "panicless-library": {
      "command": "/path/to/run-mcp-server.sh"
    }
  }
}
```

## Using with Claude Desktop

Once configured, restart Claude Desktop. You can now ask Claude to query your library:

### Example Queries

**Search for books**:
```
Search my library for books about Rust programming using user_id 1
```

**Get book details**:
```
Show me details for book ID 5 for user 1, including all reading records
```

**List current readings**:
```
What am I currently reading? (user_id: 1, status: current)
```

**Get reading statistics**:
```
Show me my reading statistics for user 1
```

**Find similar books**:
```
Find books similar to book ID 3 for user 1
```

## Available Tools

### 1. search_books

Search books in user's library.

**Parameters**:
- `user_id` (required): User ID
- `query` (optional): Search term for title/author
- `author` (optional): Filter by author
- `year` (optional): Filter by publication year

**Example**:
```json
{
  "user_id": 1,
  "query": "programming",
  "year": 2023
}
```

### 2. get_book_details

Get detailed information about a specific book including all reading records.

**Parameters**:
- `user_id` (required): User ID
- `book_id` (required): Book ID

**Example**:
```json
{
  "user_id": 1,
  "book_id": 5
}
```

### 3. list_readings

List reading records for a user.

**Parameters**:
- `user_id` (required): User ID
- `status` (optional): `"current"`, `"completed"`, or `"all"`
- `year` (optional): Filter by year

**Example**:
```json
{
  "user_id": 1,
  "status": "current"
}
```

### 4. get_reading_statistics

Get comprehensive reading statistics.

**Parameters**:
- `user_id` (required): User ID
- `year` (optional): Filter by year

**Example**:
```json
{
  "user_id": 1
}
```

### 5. find_similar_books

Find books by the same author.

**Parameters**:
- `user_id` (required): User ID
- `book_id` (required): Book ID to find similar books for

**Example**:
```json
{
  "user_id": 1,
  "book_id": 3
}
```

## Security

### Read-Only Access

The MCP server has read-only access to the database. It cannot:
- Create, update, or delete books
- Create, update, or delete readings
- Modify user data
- Execute arbitrary SQL

### User Isolation

All queries are scoped by `user_id`. The MCP server cannot access data from other users.

### Database Permissions

For production, create a read-only database user:

```sql
CREATE USER panicless_readonly WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE panicless_library TO panicless_readonly;
GRANT USAGE ON SCHEMA public TO panicless_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO panicless_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO panicless_readonly;
```

Then use this user in DATABASE_URL:
```
DATABASE_URL=postgres://panicless_readonly:secure_password@localhost:5432/panicless_library
```

## Troubleshooting

### Server not showing in Claude Desktop

1. Check Claude Desktop config file syntax (valid JSON)
2. Ensure the binary path is correct and absolute
3. Check that DATABASE_URL is accessible
4. Restart Claude Desktop completely
5. Check Claude Desktop logs:
   - macOS: `~/Library/Logs/Claude/`
   - Windows: `%APPDATA%\Claude\logs\`
   - Linux: `~/.config/Claude/logs/`

### Database connection errors

- Ensure PostgreSQL is running
- Verify DATABASE_URL is correct
- Check network connectivity
- Ensure database exists and migrations are run

### Tool execution errors

- Check that user_id exists in the database
- Ensure book_id or reading_id are valid
- Check RUST_LOG output for detailed errors

## Development

### Running in Development Mode

```bash
cargo run
```

### Testing Tools Manually

You can test tools by sending JSON-RPC messages:

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run

# Call a tool
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_books","arguments":{"user_id":1,"query":"rust"}}}' | cargo run
```

### Logging

Set `RUST_LOG` for detailed logging:

```bash
RUST_LOG=debug cargo run
```

Logs are written to stderr to not interfere with the JSON-RPC stdio protocol.

## License

Copyright (c) 2025 Federico Fuga
