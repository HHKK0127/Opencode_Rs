-- Migration 003: Performance optimization indexes
-- Purpose: Add indexes to improve query performance for common access patterns

-- User table optimization
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- File table optimization
CREATE INDEX IF NOT EXISTS idx_files_size ON files(size);
CREATE INDEX IF NOT EXISTS idx_files_created_recent ON files(uploaded_at DESC) WHERE uploaded_at > datetime('now', '-30 days');

-- PRAGMA optimizations are set at connection time
-- See src/db/optimization.rs for PRAGMA configurations
