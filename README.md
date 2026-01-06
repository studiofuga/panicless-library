# Panicless Library

A full-stack personal library management system with AI integration via Model Context Protocol (MCP).

**Track your books, manage your reading list, and query your library using Claude or Gemini.**

## Features

- **Book Catalog**: Organize your personal library with detailed book information (title, author, ISBN, publisher, etc.)
- **Reading Tracker**: Record reading sessions with start/end dates, ratings, and notes
- **Multiple Readings**: Track re-reads of your favorite books
- **Statistics Dashboard**: Visualize your reading habits and progress
- **User Management**: Multi-user support with isolated data
- **REST API**: Full-featured API for programmatic access
- **Modern Web UI**: Responsive Vue.js single-page application
- **AI Integration**: Query your library using Claude or Gemini via MCP server

## Architecture

```
panicless-library/
├── database/          # PostgreSQL migrations and seed data
├── backend/           # Rust REST API server (Axum + SQLx + JWT)
├── frontend/          # Vue.js 3 SPA (Vite + Pinia + Naive UI)
└── mcp-server/        # MCP server for AI assistant integration
```

### Technology Stack

- **Database**: PostgreSQL 16
- **Backend**: Rust (Axum, SQLx, JWT, Argon2)
- **Frontend**: Vue 3, Vite, Pinia, Vue Router, Naive UI
- **MCP Server**: Rust (JSON-RPC over stdio)
- **Development**: Docker Compose for PostgreSQL

## Quick Start

### Prerequisites

- **Rust** 1.75+ ([Install Rust](https://rustup.rs/))
- **Node.js** 18+ and npm ([Install Node](https://nodejs.org/))
- **PostgreSQL** 16+ (or Docker)
- **Git**

### 1. Clone the Repository

```bash
git clone <repository-url>
cd panicless-library
```

### 2. Start PostgreSQL

#### Option A: Using Docker (Recommended)

```bash
docker-compose up -d postgres
```

#### Option B: Using Local PostgreSQL

Ensure PostgreSQL is running and create a database:

```bash
createdb panicless_library
```

### 3. Run Database Migrations

```bash
cd database
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000001_create_users_table.sql
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000002_create_books_table.sql
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000003_create_readings_table.sql

# Optional: Load seed data for testing
psql -h localhost -U panicless -d panicless_library -f seed_data.sql
cd ..
```

### 4. Start the Backend

```bash
cd backend
cp .env.example .env
# Edit .env and set JWT_SECRET to a secure random string

cargo run
# Backend will start on http://localhost:8080
```

### 5. Start the Frontend

In a new terminal:

```bash
cd frontend
cp .env.example .env
npm install
npm run dev
# Frontend will start on http://localhost:5173
```

### 6. Access the Application

Open your browser and go to: **http://localhost:5173**

- Register a new account or login with seed data:
  - Username: `testuser`, Password: `test123`
  - Username: `demouser`, Password: `demo123`

**Note**: Seed data passwords use placeholder hashes. For production use, register via the API.

## Using the Application

### Adding Books

1. Go to **Books** page
2. Click **Add Book**
3. Fill in book details (title is required)
4. Click **Add Book**

### Tracking Reading

1. Go to **Books** page
2. Click on a book
3. Click **Start Reading**
4. When finished, go to **Readings** page
5. Click **Mark as Completed**
6. Rate the book and add notes

### Viewing Statistics

Go to **Dashboard** to see:
- Total books read
- Currently reading books
- Reading completion stats
- Average ratings
- Books read by year

## AI Integration (MCP Server)

The MCP server allows Claude or Gemini to query your library.

### Setup MCP Server

1. **Build the MCP Server**:

```bash
cd mcp-server
cargo build --release
```

2. **Configure for Claude Desktop**:

Edit your Claude Desktop config file:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Add:

```json
{
  "mcpServers": {
    "panicless-library": {
      "command": "/absolute/path/to/panicless-library/mcp-server/target/release/panicless-mcp-server",
      "env": {
        "DATABASE_URL": "postgres://panicless:panicless_dev@localhost:5432/panicless_library"
      }
    }
  }
}
```

3. **Restart Claude Desktop**

4. **Query Your Library**:

Try asking Claude:
- "Search my library for books about Rust using user_id 1"
- "What am I currently reading? (user 1)"
- "Show me my reading statistics for user 1"
- "Find books similar to book ID 3 for user 1"

See [mcp-server/README.md](mcp-server/README.md) for detailed MCP configuration.

## Development

### Project Structure

```
panicless-library/
├── database/
│   ├── migrations/          # SQL migration files
│   ├── seed_data.sql        # Test data
│   └── README.md            # Database documentation
│
├── backend/
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── config.rs        # Configuration
│   │   ├── db.rs            # Database pool
│   │   ├── errors.rs        # Error types
│   │   ├── routes.rs        # Route definitions
│   │   ├── models/          # Data models
│   │   ├── handlers/        # Request handlers
│   │   └── middleware/      # Auth middleware
│   ├── Cargo.toml
│   └── README.md            # API documentation
│
├── frontend/
│   ├── src/
│   │   ├── main.js          # Entry point
│   │   ├── App.vue          # Root component
│   │   ├── router/          # Vue Router config
│   │   ├── store/           # Pinia stores
│   │   ├── api/             # API client
│   │   ├── components/      # Vue components
│   │   └── views/           # Page views
│   ├── package.json
│   └── vite.config.js
│
└── mcp-server/
    ├── src/
    │   ├── main.rs          # Entry point
    │   ├── mcp/             # MCP protocol
    │   └── queries/         # Database queries
    ├── Cargo.toml
    └── README.md            # MCP setup guide
```

### API Documentation

See [backend/README.md](backend/README.md) for complete API documentation including:
- Authentication endpoints
- Book CRUD operations
- Reading management
- Statistics API

### Database Schema

See [database/README.md](database/README.md) for schema details and query examples.

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Manual API Testing

```bash
# Health check
curl http://localhost:8080/health

# Register
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"test1234"}'

# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test1234"}'
```

## Production Deployment

### Environment Variables

**Backend** (`backend/.env`):
```bash
DATABASE_URL=postgres://user:pass@host:5432/panicless_library
JWT_SECRET=<GENERATE-SECURE-RANDOM-STRING-64-CHARS>
RUST_LOG=info
ENVIRONMENT=production
```

**Frontend** (`frontend/.env`):
```bash
VITE_API_BASE_URL=https://api.yourdomain.com
```

### Build for Production

**Backend**:
```bash
cd backend
cargo build --release
./target/release/panicless-backend
```

**Frontend**:
```bash
cd frontend
npm run build
# Serve dist/ folder with nginx or similar
```

### Security Checklist

- [ ] Change `JWT_SECRET` to a secure random string
- [ ] Update `CORS_ALLOWED_ORIGINS` in backend
- [ ] Use HTTPS for production
- [ ] Set strong PostgreSQL password
- [ ] Create read-only DB user for MCP server
- [ ] Enable database SSL connections
- [ ] Implement rate limiting
- [ ] Regular database backups

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Troubleshooting

### Backend won't start

- Check PostgreSQL is running: `docker-compose ps`
- Verify DATABASE_URL in `backend/.env`
- Ensure migrations are run
- Check JWT_SECRET is set

### Frontend can't connect to API

- Verify backend is running on port 8080
- Check CORS settings in backend
- Check `VITE_API_BASE_URL` in `frontend/.env`

### MCP server not working

- Ensure PostgreSQL is accessible
- Check Claude Desktop config JSON syntax
- Verify absolute path to MCP server binary
- Check Claude Desktop logs
- See [mcp-server/README.md](mcp-server/README.md) for detailed troubleshooting

### Database connection errors

- Check PostgreSQL is running
- Verify credentials
- Ensure database exists
- Check network connectivity

## License

Copyright (c) 2025 Federico Fuga

## Author

Federico Fuga - fuga@studiofuga.com

---

**Built with**: Rust 🦀 | Vue.js 💚 | PostgreSQL 🐘 | MCP 🤖
