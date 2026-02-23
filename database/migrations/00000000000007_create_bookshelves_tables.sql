-- Migration: Create bookshelves and book_bookshelves tables
-- Description: User-defined tags/shelves for organizing books (many-to-many)

-- Tabella bookshelves (user-specific tags)
CREATE TABLE IF NOT EXISTS bookshelves (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT unique_bookshelf_per_user UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_bookshelves_user_id ON bookshelves(user_id);

-- Tabella junction book_bookshelves (many-to-many)
CREATE TABLE IF NOT EXISTS book_bookshelves (
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    bookshelf_id INTEGER NOT NULL REFERENCES bookshelves(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    PRIMARY KEY (book_id, bookshelf_id)
);

CREATE INDEX IF NOT EXISTS idx_book_bookshelves_book_id ON book_bookshelves(book_id);
CREATE INDEX IF NOT EXISTS idx_book_bookshelves_bookshelf_id ON book_bookshelves(bookshelf_id);

COMMENT ON TABLE bookshelves IS 'User-defined tags/shelves for organizing books';
COMMENT ON TABLE book_bookshelves IS 'Many-to-many relationship between books and bookshelves';
