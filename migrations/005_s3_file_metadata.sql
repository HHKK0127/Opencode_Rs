-- Migration: Add S3/MinIO metadata columns to files table
-- Created: 2026-05-28
-- Wave 3: S3 storage integration

-- Extend files table with S3-specific columns
ALTER TABLE files ADD COLUMN s3_path TEXT;
ALTER TABLE files ADD COLUMN s3_etag TEXT;
ALTER TABLE files ADD COLUMN s3_version_id TEXT;
ALTER TABLE files ADD COLUMN storage_type TEXT DEFAULT 'local' CHECK (storage_type IN ('local', 's3'));

-- Create indexes for S3 path lookups
CREATE INDEX IF NOT EXISTS idx_files_s3_path ON files(s3_path) WHERE storage_type = 's3';
CREATE INDEX IF NOT EXISTS idx_files_storage_type ON files(storage_type);
