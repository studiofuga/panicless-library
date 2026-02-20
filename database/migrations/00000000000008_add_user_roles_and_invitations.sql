-- Add user roles and invitation-based registration

-- Create enum type for user roles
CREATE TYPE user_role AS ENUM ('admin', 'user');

-- Add new columns to users table
ALTER TABLE users
    ADD COLUMN role user_role NOT NULL DEFAULT 'user',
    ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN invitation_token VARCHAR(96) UNIQUE,
    ADD COLUMN invitation_expires_at TIMESTAMPTZ;

-- Migrate existing data: enable all existing users, make the first user admin
UPDATE users SET enabled = true;
UPDATE users SET role = 'admin' WHERE id = (SELECT MIN(id) FROM users);

-- Indexes
CREATE INDEX idx_users_invitation_token ON users (invitation_token) WHERE invitation_token IS NOT NULL;
CREATE INDEX idx_users_role ON users (role);
CREATE INDEX idx_users_enabled ON users (enabled);
