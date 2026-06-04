-- Migration 001: Create users table
-- Purpose: Store user authentication data with unique usernames

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index for username lookup (used in login)
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
