-- Migration: Add abandoned flag and make start_date nullable in readings
-- Description: Supports abandoned readings and readings without a known start date

-- Add abandoned column
DO $$ BEGIN
    ALTER TABLE readings ADD COLUMN abandoned BOOLEAN DEFAULT FALSE NOT NULL;
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;

-- Make start_date nullable (needed for imported "read" books without a start date)
ALTER TABLE readings ALTER COLUMN start_date DROP NOT NULL;

-- Update the dates check constraint to handle nullable start_date
DO $$ BEGIN
    ALTER TABLE readings DROP CONSTRAINT reading_dates_check;
EXCEPTION
    WHEN undefined_object THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE readings ADD CONSTRAINT reading_dates_check
        CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date);
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Index for querying abandoned books
CREATE INDEX IF NOT EXISTS idx_readings_abandoned ON readings(user_id, abandoned) WHERE abandoned = TRUE;

COMMENT ON COLUMN readings.abandoned IS 'Indicates if the reading was abandoned (book started but not finished)';
