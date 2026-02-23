-- Add user roles and invitation-based registration

-- Create enum type for user roles
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('admin', 'user');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Add new columns to users table
DO $$ BEGIN
    ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'user';
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE users ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT false;
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE users ADD COLUMN invitation_token VARCHAR(96) UNIQUE;
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;
DO $$ BEGIN
    ALTER TABLE users ADD COLUMN invitation_expires_at TIMESTAMPTZ;
EXCEPTION
    WHEN duplicate_column THEN NULL;
END $$;

-- Migrate existing data: enable all existing users, make the first user admin
UPDATE users SET enabled = true;
UPDATE users SET role = 'admin' WHERE id = (SELECT MIN(id) FROM users);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_users_invitation_token ON users (invitation_token) WHERE invitation_token IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_role ON users (role);
CREATE INDEX IF NOT EXISTS idx_users_enabled ON users (enabled);
