# Panicless Backend

REST API server for Panicless Library, built with Rust, Axum, and PostgreSQL.

## Features

- **Authentication**: JWT-based with Argon2 password hashing
- **Authorization**: User-scoped data access
- **Database**: PostgreSQL with SQLx
- **Validation**: Request validation with validator crate
- **Error Handling**: Comprehensive error types with proper HTTP status codes
- **Logging**: Structured logging with tracing
- **CORS**: Configured for frontend integration

## Technology Stack

- **Web Framework**: Axum 0.7
- **Database**: SQLx 0.7 with PostgreSQL
- **Authentication**: JWT (jsonwebtoken) + Argon2
- **Async Runtime**: Tokio
- **Serialization**: Serde
- **Validation**: validator
- **Logging**: tracing + tracing-subscriber

## Getting Started

### Prerequisites

- Rust 1.85+ (edition 2021, supports edition 2024 dependencies)
- PostgreSQL 16+ (or use Docker Compose)
- Environment variables configured (see `.env.example`)

### Setup

1. **Copy environment variables**:
   ```bash
   cd backend
   cp .env.example .env
   ```

2. **Edit `.env` file**:
   - Set `JWT_SECRET` to a secure random string (64+ characters)
   - Configure `DATABASE_URL` if not using default
   - Adjust `CORS_ALLOWED_ORIGINS` for your frontend

3. **Start PostgreSQL** (if using Docker):
   ```bash
   cd ..
   docker-compose up -d postgres
   ```

4. **Run database migrations**:
   ```bash
   cd ../database
   psql -h localhost -U panicless -d panicless_library -f migrations/00000000000001_create_users_table.sql
   psql -h localhost -U panicless -d panicless_library -f migrations/00000000000002_create_books_table.sql
   psql -h localhost -U panicless -d panicless_library -f migrations/00000000000003_create_readings_table.sql
   ```

5. **Optional - Load seed data**:
   ```bash
   psql -h localhost -U panicless -d panicless_library -f seed_data.sql
   ```

### Running the Server

#### Development mode:
```bash
cd backend
cargo run
```

#### Production build:
```bash
cargo build --release
./target/release/panicless-backend
```

The server will start on `http://127.0.0.1:8080` by default.

### Testing

```bash
cargo test
```

## API Endpoints

### Authentication (`/api/auth`)

#### POST `/api/auth/register`
Register a new user account.

**Request**:
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "secure_password_123",
  "full_name": "John Doe"
}
```

**Response** (201 Created):
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": 1,
    "username": "johndoe",
    "email": "john@example.com",
    "full_name": "John Doe",
    "created_at": "2025-01-06T10:00:00Z"
  }
}
```

#### POST `/api/auth/login`
Login with username and password.

**Request**:
```json
{
  "username": "johndoe",
  "password": "secure_password_123"
}
```

**Response** (200 OK): Same as register

#### POST `/api/auth/refresh`
Refresh access token using refresh token.

**Request**:
```json
{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

**Response** (200 OK): Same as login

#### GET `/api/auth/me`
Get current user information.

**Headers**: `Authorization: Bearer <access_token>`

**Response** (200 OK):
```json
{
  "id": 1,
  "username": "johndoe",
  "email": "john@example.com",
  "full_name": "John Doe",
  "created_at": "2025-01-06T10:00:00Z"
}
```

### Books (`/api/books`)

All book endpoints require authentication (`Authorization: Bearer <access_token>`).

#### GET `/api/books`
List all books for the authenticated user.

**Query Parameters**:
- `search`: Search in title and author (optional)
- `author`: Filter by author (optional)
- `year`: Filter by publication year (optional)
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20)

**Response** (200 OK):
```json
[
  {
    "id": 1,
    "user_id": 1,
    "title": "The Rust Programming Language",
    "author": "Steve Klabnik, Carol Nichols",
    "edition": "2nd",
    "isbn": "978-1-7185-0044-0",
    "publication_year": 2023,
    "publisher": "No Starch Press",
    "pages": 552,
    "language": "English",
    "description": "The official book on Rust...",
    "cover_image_url": null,
    "created_at": "2025-01-06T10:00:00Z",
    "updated_at": "2025-01-06T10:00:00Z"
  }
]
```

#### POST `/api/books`
Create a new book.

**Request**:
```json
{
  "title": "Clean Code",
  "author": "Robert C. Martin",
  "edition": "1st",
  "isbn": "978-0-1323-5088-4",
  "publication_year": 2008,
  "publisher": "Prentice Hall",
  "pages": 464,
  "language": "English",
  "description": "A Handbook of Agile Software Craftsmanship"
}
```

**Response** (201 Created): Created book object

#### GET `/api/books/:id`
Get a specific book by ID.

**Response** (200 OK): Book object

#### PUT `/api/books/:id`
Update a book (partial updates allowed).

**Request**: Same as create, all fields optional

**Response** (200 OK): Updated book object

#### DELETE `/api/books/:id`
Delete a book.

**Response** (200 OK):
```json
{
  "message": "Book deleted successfully"
}
```

#### GET `/api/books/:id/readings`
Get all reading records for a specific book.

**Response** (200 OK): Array of reading objects

### Readings (`/api/readings`)

All reading endpoints require authentication.

#### GET `/api/readings`
List all readings for the authenticated user.

**Query Parameters**:
- `status`: Filter by status (`current`, `completed`, `all`) (optional)
- `book_id`: Filter by book ID (optional)
- `year`: Filter by year (optional)
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20)

**Response** (200 OK):
```json
[
  {
    "id": 1,
    "user_id": 1,
    "book_id": 1,
    "start_date": "2025-01-01",
    "end_date": "2025-02-15",
    "rating": 5,
    "notes": "Excellent introduction to Rust",
    "created_at": "2025-01-01T10:00:00Z",
    "updated_at": "2025-02-15T10:00:00Z",
    "book_title": "The Rust Programming Language",
    "book_author": "Steve Klabnik, Carol Nichols"
  }
]
```

#### POST `/api/readings`
Create a new reading record.

**Request**:
```json
{
  "book_id": 1,
  "start_date": "2025-01-01",
  "end_date": null,
  "rating": null,
  "notes": "Started reading this book"
}
```

**Response** (201 Created): Created reading object

#### GET `/api/readings/:id`
Get a specific reading by ID.

**Response** (200 OK): Reading object

#### PUT `/api/readings/:id`
Update a reading record.

**Request**: Same as create, all fields optional

**Response** (200 OK): Updated reading object

#### PATCH `/api/readings/:id/complete`
Mark a reading as completed.

**Request**:
```json
{
  "end_date": "2025-02-15",
  "rating": 5
}
```

**Response** (200 OK): Updated reading object

#### DELETE `/api/readings/:id`
Delete a reading record.

**Response** (200 OK):
```json
{
  "message": "Reading deleted successfully"
}
```

#### GET `/api/readings/stats`
Get reading statistics for the authenticated user.

**Response** (200 OK):
```json
{
  "total_readings": 10,
  "completed_readings": 8,
  "current_readings": 2,
  "total_books_read": 7,
  "average_rating": 4.5,
  "books_by_year": [
    { "year": 2025, "count": 3 },
    { "year": 2024, "count": 5 }
  ]
}
```

### Users (`/api/users`)

#### GET `/api/users/:id`
Get user profile (own profile only).

**Response** (200 OK): User object

#### PUT `/api/users/:id`
Update user profile (own profile only).

**Request**:
```json
{
  "email": "newemail@example.com",
  "full_name": "New Name"
}
```

**Response** (200 OK): Updated user object

#### DELETE `/api/users/:id`
Delete user account (own account only).

**Response** (200 OK):
```json
{
  "message": "User deleted successfully"
}
```

### Health Check

#### GET `/health`
Health check endpoint (no authentication required).

**Response** (200 OK): `OK`

## Error Responses

All errors follow this format:

```json
{
  "error": "Short error message",
  "message": "Detailed error description"
}
```

**Common Status Codes**:
- `400 Bad Request`: Validation error or malformed request
- `401 Unauthorized`: Missing or invalid authentication token
- `403 Forbidden`: User doesn't have permission to access resource
- `404 Not Found`: Resource doesn't exist
- `409 Conflict`: Duplicate resource (username/email already exists)
- `500 Internal Server Error`: Server error

## Security

### Password Hashing
Passwords are hashed using Argon2id algorithm, the winner of the Password Hashing Competition and recommended by OWASP.

### JWT Tokens
- **Access Token**: Short-lived (1 hour), used for API requests
- **Refresh Token**: Long-lived (7 days), used to obtain new access tokens
- Tokens include user ID and username in claims

### Data Isolation
- All user data is scoped by `user_id`
- Users can only access their own books and readings
- Database triggers prevent cross-user data access

### CORS
Configure `CORS_ALLOWED_ORIGINS` in `.env` to restrict allowed origins in production.

## Development

### Project Structure

```
backend/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library exports
│   ├── config.rs         # Configuration management
│   ├── db.rs             # Database connection pool
│   ├── errors.rs         # Error types
│   ├── routes.rs         # Route definitions
│   ├── models/           # Data models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── book.rs
│   │   └── reading.rs
│   ├── handlers/         # Request handlers
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── books.rs
│   │   └── readings.rs
│   └── middleware/       # Middleware
│       ├── mod.rs
│       └── auth.rs       # JWT verification
├── Cargo.toml
├── .env.example
└── README.md
```

### Adding New Endpoints

1. Add handler function in appropriate `handlers/*.rs` file
2. Define route in `routes.rs`
3. Add to protected or public routes as needed
4. Update this README with API documentation

### Database Queries

All database queries use SQLx with compile-time verification. To enable:

```bash
# Set DATABASE_URL
export DATABASE_URL="postgres://panicless:panicless_dev@localhost:5432/panicless_library"

# Run cargo sqlx prepare to cache query metadata
cargo sqlx prepare
```

## Troubleshooting

### Connection refused to PostgreSQL

Ensure PostgreSQL is running:
```bash
docker-compose up -d postgres
# OR
systemctl status postgresql
```

### JWT token errors

- Ensure `JWT_SECRET` is set in `.env`
- Check that the token hasn't expired
- Verify `Authorization: Bearer <token>` header format

### Database errors

- Run migrations in correct order
- Check `DATABASE_URL` is correct
- Ensure database exists and user has permissions

## License

Copyright (c) 2025 Federico Fuga
