-- Migration: Create readings table
-- Description: Tracks reading activity - links users to books with start/end dates

CREATE TABLE IF NOT EXISTS readings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE,  -- NULL means currently reading
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    -- Ensure end_date is after start_date when both are set
    CONSTRAINT reading_dates_check CHECK (end_date IS NULL OR end_date >= start_date)
);

-- Create indexes for faster lookups and queries
CREATE INDEX IF NOT EXISTS idx_readings_user_id ON readings(user_id);
CREATE INDEX IF NOT EXISTS idx_readings_book_id ON readings(book_id);
CREATE INDEX IF NOT EXISTS idx_readings_start_date ON readings(start_date);
CREATE INDEX IF NOT EXISTS idx_readings_end_date ON readings(end_date);
CREATE INDEX IF NOT EXISTS idx_readings_user_book ON readings(user_id, book_id);

-- Prevent multiple concurrent "currently reading" records for the same book
-- (only one reading with NULL end_date allowed per user+book combination)
CREATE UNIQUE INDEX IF NOT EXISTS idx_readings_no_overlap
    ON readings (user_id, book_id)
    WHERE end_date IS NULL;

-- Create trigger to automatically update updated_at timestamp
CREATE TRIGGER update_readings_updated_at
    BEFORE UPDATE ON readings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add function to validate that user owns the book they're creating a reading for
-- This prevents users from creating readings for books that don't belong to them
CREATE OR REPLACE FUNCTION validate_reading_ownership()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if the book belongs to the user
    IF NOT EXISTS (
        SELECT 1 FROM books
        WHERE id = NEW.book_id AND user_id = NEW.user_id
    ) THEN
        RAISE EXCEPTION 'Cannot create reading: book % does not belong to user %',
            NEW.book_id, NEW.user_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to validate ownership before INSERT or UPDATE
CREATE TRIGGER validate_reading_ownership_trigger
    BEFORE INSERT OR UPDATE ON readings
    FOR EACH ROW
    EXECUTE FUNCTION validate_reading_ownership();

-- Add comments for documentation
COMMENT ON TABLE readings IS 'Tracks user reading activity with start/end dates and ratings';
COMMENT ON COLUMN readings.id IS 'Primary key, auto-incrementing reading record identifier';
COMMENT ON COLUMN readings.user_id IS 'Foreign key to users table - reader';
COMMENT ON COLUMN readings.book_id IS 'Foreign key to books table - book being read';
COMMENT ON COLUMN readings.start_date IS 'Date when user started reading the book';
COMMENT ON COLUMN readings.end_date IS 'Date when user finished reading (NULL if still reading)';
COMMENT ON COLUMN readings.rating IS 'User rating of the book (1-5 stars)';
COMMENT ON COLUMN readings.notes IS 'User notes about their reading experience';
