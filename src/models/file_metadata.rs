use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadataRegisterRequest {
    pub filename: String,
    pub s3_path: String,
    pub s3_etag: String,
    pub s3_version_id: Option<String>,
    pub size: i64,
    pub mime_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileMetadataResponse {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub s3_path: String,
    pub s3_etag: String,
    pub s3_version_id: Option<String>,
    pub storage_type: String,
    pub download_url: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3UploadCompleteRequest {
    pub s3_path: String,
    pub s3_etag: String,
    pub filename: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
