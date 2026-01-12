-- Create connectors table for storing encrypted AI provider tokens
-- Migration: 00000000000004_create_connectors_table

CREATE TABLE IF NOT EXISTS connectors (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    encrypted_token TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true NOT NULL,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT unique_active_connector UNIQUE (user_id, provider)
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_connectors_user_id ON connectors(user_id);
CREATE INDEX IF NOT EXISTS idx_connectors_provider ON connectors(provider);
CREATE INDEX IF NOT EXISTS idx_connectors_user_provider ON connectors(user_id, provider);

-- Create trigger to automatically update updated_at timestamp
CREATE TRIGGER update_connectors_updated_at
    BEFORE UPDATE ON connectors
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add validation for provider enum
ALTER TABLE connectors
    ADD CONSTRAINT check_provider_type
    CHECK (provider IN ('anthropic', 'gemini', 'chatgpt'));

-- Add comments for documentation
COMMENT ON TABLE connectors IS 'Stores encrypted API tokens for AI provider integrations';
COMMENT ON COLUMN connectors.id IS 'Primary key, auto-incrementing connector identifier';
COMMENT ON COLUMN connectors.user_id IS 'Foreign key to users table - token owner';
COMMENT ON COLUMN connectors.provider IS 'AI provider name: anthropic, gemini, or chatgpt';
COMMENT ON COLUMN connectors.encrypted_token IS 'Encrypted API token/key for the provider';
COMMENT ON COLUMN connectors.is_active IS 'Whether this connector is currently active';
COMMENT ON COLUMN connectors.last_used_at IS 'Last time this connector was used for an API call';
COMMENT ON COLUMN connectors.created_at IS 'Timestamp when connector was created';
COMMENT ON COLUMN connectors.updated_at IS 'Timestamp when connector was last updated';
