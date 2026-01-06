-- Migration: Create books table
-- Description: Stores book catalog information for each user's library

CREATE TABLE IF NOT EXISTS books (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    author VARCHAR(255),
    edition VARCHAR(50),
    isbn VARCHAR(17),  -- ISBN-13 format: XXX-X-XX-XXXXXX-X
    publication_year INTEGER CHECK (publication_year >= 1000 AND publication_year <= 9999),
    publisher VARCHAR(255),
    pages INTEGER CHECK (pages > 0),
    language VARCHAR(50),
    description TEXT,
    cover_image_url VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create indexes for faster lookups and queries
CREATE INDEX IF NOT EXISTS idx_books_user_id ON books(user_id);
CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author);
CREATE INDEX IF NOT EXISTS idx_books_isbn ON books(isbn);
CREATE INDEX IF NOT EXISTS idx_books_publication_year ON books(publication_year);

-- Create trigger to automatically update updated_at timestamp
CREATE TRIGGER update_books_updated_at
    BEFORE UPDATE ON books
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE books IS 'Stores book information in user libraries';
COMMENT ON COLUMN books.id IS 'Primary key, auto-incrementing book identifier';
COMMENT ON COLUMN books.user_id IS 'Foreign key to users table - book owner';
COMMENT ON COLUMN books.title IS 'Book title';
COMMENT ON COLUMN books.author IS 'Book author(s)';
COMMENT ON COLUMN books.edition IS 'Book edition (e.g., "1st", "2nd", "Revised")';
COMMENT ON COLUMN books.isbn IS 'International Standard Book Number (ISBN-13 format)';
COMMENT ON COLUMN books.publication_year IS 'Year of publication';
COMMENT ON COLUMN books.publisher IS 'Publisher name';
COMMENT ON COLUMN books.pages IS 'Number of pages';
COMMENT ON COLUMN books.language IS 'Language code or name (e.g., "en", "it", "English", "Italian")';
COMMENT ON COLUMN books.description IS 'Book description or synopsis';
COMMENT ON COLUMN books.cover_image_url IS 'URL to book cover image';
