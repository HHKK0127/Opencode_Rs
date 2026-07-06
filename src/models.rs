use serde::{Deserialize, Serialize};

// ============================================================
// 統一API Response Envelope (Wave 6: フロントエンド統合)
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: String,         // "success" or "error"
    pub data: Option<T>,        // Response data (null on error)
    pub error: Option<ApiError>, // Error details (null on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>, // ISO 8601 timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,           // Error code (e.g., "UNAUTHORIZED", "VALIDATION_ERROR")
    pub message: String,        // User-facing message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>, // Technical details (dev only)
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            }),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    pub fn error_with_details(code: &str, message: &str, details: &str) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: Some(details.to_string()),
            }),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

// Generic empty response
#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyResponse {
    pub message: String,
}

// ============================================================
// 既存のモデル定義（互換性維持）
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub id: String,
    pub filename: String,
    pub size: i64,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

// Wave 3: S3/MinIO Presigned URLs
#[derive(Debug, Deserialize)]
pub struct PresignedPutRequest {
    pub filename: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PresignedGetRequest {
    pub file_id: String,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PresignedUrlResponse {
    pub presigned_url: String,
    pub expires_in_seconds: u64,
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct PresignedUrlError {
    pub error: String,
    pub details: String,
}

// Module declarations for organized models
pub mod file_metadata;

// Wave 2B: Chunked Upload API
#[derive(Debug, Deserialize)]
pub struct ChunkedUploadInitRequest {
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub chunk_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ChunkedUploadInitResponse {
    pub session_id: String,
    pub chunk_size: i64,
    pub total_chunks: i64,
}

#[derive(Debug, Serialize)]
pub struct ChunkedUploadProgressResponse {
    pub session_id: String,
    pub uploaded_size: i64,
    pub total_size: i64,
    pub progress_percent: f32,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ChunkedUploadCompleteResponse {
    pub file_id: String,
    pub filename: String,
    pub size: i64,
    pub mime_type: String,
    pub checksum: String,
    pub created_at: String,
}
