# Panicless Library - Development Notes

## Project Structure

- **backend/**: Axum-based REST API server (Rust)
- **panicless-mcp-lib/**: Shared MCP protocol library with tool definitions and database queries
- **mcp-server/**: Standalone stdio-based MCP server — this is only a thin interface (stdin/stdout transport) over `panicless-mcp-lib`. All MCP tool logic lives in `panicless-mcp-lib`.
- **mcp-oauth/**: Reusable OAuth 2.0 library for MCP servers
- **database/migrations/**: SQL migration files (applied via `sqlx::migrate!()` at backend startup)
- **frontend/**: Web UI

## Key Conventions

- Database migrations are idempotent (safe to re-run on existing DBs)
- Migrations are auto-applied by the backend on startup (`AUTO_MIGRATE=true` by default)
- The backend also exposes MCP tools via SSE at `/mcp` (using `panicless-mcp-lib`)
