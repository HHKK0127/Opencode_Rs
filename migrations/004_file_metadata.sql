-- Migration: Add file metadata and upload sessions tables
-- Created: 2026-05-28
-- Wave 2: File processing implementation

-- Extend files table with additional metadata columns
ALTER TABLE files ADD COLUMN user_id TEXT;
ALTER TABLE files ADD COLUMN original_name TEXT;
ALTER TABLE files ADD COLUMN mime_type TEXT;
ALTER TABLE files ADD COLUMN checksum TEXT;
ALTER TABLE files ADD COLUMN description TEXT;
ALTER TABLE files ADD COLUMN tags TEXT;
ALTER TABLE files ADD COLUMN is_public BOOLEAN DEFAULT FALSE;
ALTER TABLE files ADD COLUMN expires_at TIMESTAMP;
ALTER TABLE files ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id);
CREATE INDEX IF NOT EXISTS idx_files_uploaded_at ON files(uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_files_size ON files(size);
CREATE INDEX IF NOT EXISTS idx_files_mime_type ON files(mime_type);
CREATE INDEX IF NOT EXISTS idx_files_is_public ON files(is_public) WHERE is_public = TRUE;

-- Partial index for expiring files
CREATE INDEX IF NOT EXISTS idx_files_expires_at ON files(expires_at)
    WHERE expires_at IS NOT NULL;

-- Full-text search table (commented out for Wave 2 - uncomment for Wave 3)
-- CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
--     filename,
--     description,
--     content='files',
--     content_rowid='id'
-- );

-- Upload sessions table for tracking large file uploads
CREATE TABLE IF NOT EXISTS upload_sessions(
    id TEXT PRIMARY KEY,
    file_id TEXT,
    user_id TEXT NOT NULL,
    total_size INTEGER NOT NULL,
    uploaded_size INTEGER DEFAULT 0,
    chunk_size INTEGER DEFAULT 1048576,
    status TEXT CHECK (status IN ('pending', 'uploading', 'completed', 'failed')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_upload_sessions_user_id ON upload_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_upload_sessions_status ON upload_sessions(status);
CREATE INDEX IF NOT EXISTS idx_upload_sessions_file_id ON upload_sessions(file_id);

-- ---- DOWN (Rollback) ----
-- DROP TABLE IF EXISTS upload_sessions;
-- DROP TABLE IF EXISTS files;
