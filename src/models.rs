use serde::{Deserialize, Serialize};

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
