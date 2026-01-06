# Database Schema

PostgreSQL database schema for Panicless Library.

## Schema Overview

The database consists of three main tables:

1. **users** - User accounts for authentication and data isolation
2. **books** - Book catalog, each book belongs to a user
3. **readings** - Reading tracking (many-to-many relationship between users and books)

## Running Migrations

### Option 1: Using Docker Compose (Automatic)

Migrations are automatically applied when starting PostgreSQL via docker-compose:

```bash
docker-compose up -d postgres
```

The migrations in `/database/migrations/` are mounted to `/docker-entrypoint-initdb.d/` and run in alphabetical order on first startup.

### Option 2: Manual Migration

If you already have PostgreSQL running:

```bash
# Run migrations in order
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000001_create_users_table.sql
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000002_create_books_table.sql
psql -h localhost -U panicless -d panicless_library -f migrations/00000000000003_create_readings_table.sql
```

### Option 3: All at Once

```bash
cd database
cat migrations/*.sql | psql -h localhost -U panicless -d panicless_library
```

## Seed Data

To populate the database with test data:

```bash
psql -h localhost -U panicless -d panicless_library -f seed_data.sql
```

This creates:
- 2 test users (testuser, demouser)
- 9 sample books
- 7 reading records

**Note**: The seed data includes placeholder password hashes. Use the backend API `/register` endpoint to create users with proper Argon2 hashed passwords.

## Database Connection

Default connection parameters (from docker-compose.yml):

```
Host: localhost
Port: 5432
Database: panicless_library
Username: panicless
Password: panicless_dev
```

Connection string:
```
postgres://panicless:panicless_dev@localhost:5432/panicless_library
```

## Schema Details

### Users Table

| Column        | Type      | Description                    |
|---------------|-----------|--------------------------------|
| id            | SERIAL    | Primary key                    |
| username      | VARCHAR   | Unique username for login      |
| email         | VARCHAR   | Unique email address           |
| password_hash | VARCHAR   | Argon2 hashed password         |
| full_name     | VARCHAR   | Optional display name          |
| created_at    | TIMESTAMP | Record creation time           |
| updated_at    | TIMESTAMP | Last update time               |

**Indexes**: username, email

### Books Table

| Column            | Type      | Description                    |
|-------------------|-----------|--------------------------------|
| id                | SERIAL    | Primary key                    |
| user_id           | INTEGER   | FK to users (owner)            |
| title             | VARCHAR   | Book title                     |
| author            | VARCHAR   | Book author(s)                 |
| edition           | VARCHAR   | Edition info                   |
| isbn              | VARCHAR   | ISBN-13 code                   |
| publication_year  | INTEGER   | Year published                 |
| publisher         | VARCHAR   | Publisher name                 |
| pages             | INTEGER   | Number of pages                |
| language          | VARCHAR   | Language code/name             |
| description       | TEXT      | Book description               |
| cover_image_url   | VARCHAR   | URL to cover image             |
| created_at        | TIMESTAMP | Record creation time           |
| updated_at        | TIMESTAMP | Last update time               |

**Indexes**: user_id, title, author, isbn, publication_year

**Constraints**:
- ON DELETE CASCADE (when user is deleted, their books are deleted)
- publication_year between 1000-9999
- pages > 0

### Readings Table

| Column     | Type      | Description                    |
|------------|-----------|--------------------------------|
| id         | SERIAL    | Primary key                    |
| user_id    | INTEGER   | FK to users (reader)           |
| book_id    | INTEGER   | FK to books (book being read)  |
| start_date | DATE      | Date started reading           |
| end_date   | DATE      | Date finished (NULL if ongoing)|
| rating     | INTEGER   | User rating (1-5)              |
| notes      | TEXT      | User notes                     |
| created_at | TIMESTAMP | Record creation time           |
| updated_at | TIMESTAMP | Last update time               |

**Indexes**: user_id, book_id, start_date, end_date, (user_id, book_id)

**Constraints**:
- ON DELETE CASCADE (when user or book deleted, readings are deleted)
- end_date >= start_date
- rating between 1-5
- Only one "currently reading" record per user+book (unique index where end_date IS NULL)
- User can only create readings for books they own (trigger validation)

## Automatic Features

### Timestamp Management

All tables have `created_at` and `updated_at` timestamps that are:
- Automatically set on INSERT
- Automatically updated on UPDATE (via trigger)

### Data Isolation

Users can only access their own books and readings:
- Books have `user_id` foreign key
- Readings have validation trigger to ensure book ownership
- Application layer enforces user-scoped queries

### Cascade Deletes

When a user is deleted:
- All their books are deleted (CASCADE)
- All their reading records are deleted (CASCADE)

When a book is deleted:
- All reading records for that book are deleted (CASCADE)

## Querying Examples

### Get all books for a user
```sql
SELECT * FROM books WHERE user_id = 1 ORDER BY title;
```

### Get currently reading books
```sql
SELECT b.*, r.start_date, r.notes
FROM books b
JOIN readings r ON b.id = r.book_id
WHERE r.user_id = 1 AND r.end_date IS NULL;
```

### Get reading statistics
```sql
SELECT
    COUNT(DISTINCT book_id) AS books_read,
    COUNT(*) AS total_readings,
    AVG(rating) AS avg_rating
FROM readings
WHERE user_id = 1 AND end_date IS NOT NULL;
```

### Books read in a specific year
```sql
SELECT b.*, r.start_date, r.end_date, r.rating
FROM books b
JOIN readings r ON b.id = r.book_id
WHERE r.user_id = 1
  AND EXTRACT(YEAR FROM r.end_date) = 2025
ORDER BY r.end_date DESC;
```
