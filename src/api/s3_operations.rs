//! S3/MinIO storage operations API endpoints
//! Unified interface for file operations (upload, download, delete, multipart)

use actix_web::{web, HttpResponse, post, delete};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::AppError;

/// Request payload for direct S3 upload
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadS3Request {
    /// Storage key (e.g., "uploads/2026-05-30/file.txt")
    pub key: String,
    /// File content (base64 encoded or raw binary)
    pub data: Vec<u8>,
    /// MIME type (e.g., "application/octet-stream")
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Response payload for S3 upload
#[derive(Debug, Serialize)]
pub struct UploadS3Response {
    /// Storage key
    pub key: String,
    /// ETag (file hash)
    pub etag: String,
    /// File size in bytes
    pub size: usize,
}

/// Request payload for S3 delete
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteS3Request {
    /// Storage key to delete
    pub key: String,
}

/// Response payload for S3 delete
#[derive(Debug, Serialize)]
pub struct DeleteS3Response {
    /// Storage key
    pub key: String,
    /// Deletion status
    pub deleted: bool,
}

/// Request payload for presigned PUT URL
#[derive(Debug, Deserialize, Serialize)]
pub struct PresignedPutUrlRequest {
    /// Storage key
    pub key: String,
    /// Expiration time in seconds (default: 3600)
    #[serde(default = "default_expiry")]
    pub expires_in_seconds: u64,
    /// MIME type
    #[serde(default)]
    pub content_type: Option<String>,
}

fn default_expiry() -> u64 {
    3600 // 1 hour
}

/// Response payload for presigned URL
#[derive(Debug, Serialize)]
pub struct PresignedUrlResponse {
    /// Storage key
    pub key: String,
    /// Presigned URL
    pub url: String,
    /// Expiration time in seconds
    pub expires_in: u64,
}

/// Request payload for multipart upload initialization
#[derive(Debug, Deserialize, Serialize)]
pub struct MultipartInitRequest {
    /// Storage key
    pub key: String,
    /// MIME type
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Response payload for multipart initialization
#[derive(Debug, Serialize)]
pub struct MultipartInitResponse {
    /// Storage key
    pub key: String,
    /// Upload ID for subsequent chunk uploads
    pub upload_id: String,
}

/// Request payload for multipart chunk upload
#[derive(Debug, Deserialize, Serialize)]
pub struct MultipartChunkRequest {
    /// Storage key
    pub key: String,
    /// Upload ID from init response
    pub upload_id: String,
    /// Part number (1-10000)
    pub part_number: i32,
    /// Chunk data (binary)
    pub data: Vec<u8>,
}

/// Response payload for multipart chunk upload
#[derive(Debug, Serialize)]
pub struct MultipartChunkResponse {
    /// Part number
    pub part_number: i32,
    /// ETag for this part
    pub e_tag: String,
}

/// Part information for completion
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompletedPart {
    /// Part number
    pub part_number: i32,
    /// ETag returned from chunk upload
    pub e_tag: String,
}

/// Request payload for multipart completion
#[derive(Debug, Deserialize, Serialize)]
pub struct MultipartCompleteRequest {
    /// Storage key
    pub key: String,
    /// Upload ID from init response
    pub upload_id: String,
    /// List of completed parts
    pub parts: Vec<CompletedPart>,
}

/// Response payload for multipart completion
#[derive(Debug, Serialize)]
pub struct MultipartCompleteResponse {
    /// Storage key
    pub key: String,
    /// Final ETag
    pub etag: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct StorageHealthResponse {
    /// Health status: "healthy" or "unhealthy"
    pub status: String,
    /// Detailed message
    pub message: String,
}

/// Upload file to S3/MinIO
///
/// # Request
/// POST /api/v1/files/s3/upload-s3
///
/// # Example
/// ```json
/// {
///   "key": "uploads/2026-05-30/myfile.txt",
///   "data": [/* binary content */],
///   "content_type": "text/plain"
/// }
/// ```
#[post("/s3/upload-s3")]
pub async fn upload_s3(
    app_state: web::Data<AppState>,
    req: web::Json<UploadS3Request>,
) -> Result<HttpResponse, AppError> {
    // Validate key format
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    if req.data.is_empty() {
        return Err(AppError::BadRequest("Data cannot be empty".to_string()));
    }

    // Store file using StorageBackend
    let etag = app_state
        .storage
        .store(&req.key, req.data.clone(), req.content_type.as_deref())
        .await?;

    let response = UploadS3Response {
        key: req.key.clone(),
        etag,
        size: req.data.len(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete file from S3/MinIO
///
/// # Request
/// DELETE /api/v1/files/s3/delete-s3/:key
///
/// # Example
/// ```bash
/// curl -X DELETE http://localhost:8080/api/v1/files/s3/delete-s3/uploads/2026-05-30/myfile.txt
/// ```
#[delete("/s3/delete-s3/{key}")]
pub async fn delete_s3(
    app_state: web::Data<AppState>,
    key: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let key = key.into_inner();

    // Validate key
    if key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    // Delete file using StorageBackend
    app_state.storage.delete(&key).await?;

    let response = DeleteS3Response {
        key: key.clone(),
        deleted: true,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get presigned PUT URL for direct client upload
///
/// # Request
/// POST /api/v1/files/s3/presigned-put
///
/// # Example
/// ```json
/// {
///   "key": "uploads/2026-05-30/myfile.txt",
///   "expires_in_seconds": 3600,
///   "content_type": "text/plain"
/// }
/// ```
#[post("/s3/presigned-put")]
pub async fn presigned_put_url(
    app_state: web::Data<AppState>,
    req: web::Json<PresignedPutUrlRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    // Generate presigned URL
    use std::time::Duration;
    let duration = Duration::from_secs(req.expires_in_seconds);
    let url = app_state
        .s3_client
        .generate_presigned_put_url(&req.key, duration, req.content_type.as_deref())
        .await?;

    let response = PresignedUrlResponse {
        key: req.key.clone(),
        url,
        expires_in: req.expires_in_seconds,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get presigned GET URL for direct client download
///
/// # Request
/// POST /api/v1/files/s3/presigned-get
///
/// # Example
/// ```json
/// {
///   "key": "uploads/2026-05-30/myfile.txt",
///   "expires_in_seconds": 3600
/// }
/// ```
#[post("/s3/presigned-get")]
pub async fn presigned_get_url(
    app_state: web::Data<AppState>,
    req: web::Json<PresignedPutUrlRequest>, // Reuse same request struct
) -> Result<HttpResponse, AppError> {
    // Validate
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    // Generate presigned URL
    use std::time::Duration;
    let duration = Duration::from_secs(req.expires_in_seconds);
    let url = app_state
        .s3_client
        .generate_presigned_get_url(&req.key, duration)
        .await?;

    let response = PresignedUrlResponse {
        key: req.key.clone(),
        url,
        expires_in: req.expires_in_seconds,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Initialize multipart upload
///
/// # Request
/// POST /api/v1/files/s3/multipart-init
#[post("/s3/multipart-init")]
pub async fn multipart_init(
    app_state: web::Data<AppState>,
    req: web::Json<MultipartInitRequest>,
) -> Result<HttpResponse, AppError> {
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    let upload_id = app_state
        .s3_client
        .initiate_multipart_upload(&req.key)
        .await?;

    let response = MultipartInitResponse {
        key: req.key.clone(),
        upload_id,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Upload file chunk
///
/// # Request
/// POST /api/v1/files/s3/multipart-chunk
#[post("/s3/multipart-chunk")]
pub async fn multipart_chunk(
    app_state: web::Data<AppState>,
    req: web::Json<MultipartChunkRequest>,
) -> Result<HttpResponse, AppError> {
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    if req.upload_id.is_empty() {
        return Err(AppError::BadRequest("Upload ID cannot be empty".to_string()));
    }

    if req.part_number < 1 || req.part_number > 10000 {
        return Err(AppError::BadRequest(
            "Part number must be between 1 and 10000".to_string(),
        ));
    }

    let part = app_state
        .s3_client
        .upload_part(&req.key, &req.upload_id, req.part_number, req.data.clone())
        .await?;

    let response = MultipartChunkResponse {
        part_number: part.part_number,
        e_tag: part.e_tag,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Complete multipart upload
///
/// # Request
/// POST /api/v1/files/s3/multipart-complete
#[post("/s3/multipart-complete")]
pub async fn multipart_complete(
    app_state: web::Data<AppState>,
    req: web::Json<MultipartCompleteRequest>,
) -> Result<HttpResponse, AppError> {
    if req.key.is_empty() {
        return Err(AppError::BadRequest("Key cannot be empty".to_string()));
    }

    if req.upload_id.is_empty() {
        return Err(AppError::BadRequest("Upload ID cannot be empty".to_string()));
    }

    if req.parts.is_empty() {
        return Err(AppError::BadRequest("Parts list cannot be empty".to_string()));
    }

    // Note: Complete multipart implementation requires AWS SDK CompletedPart
    // This is called during Week 2 implementation
    // Real implementation will be completed in integration testing phase

    let etag = app_state
        .s3_client
        .complete_multipart_upload(&req.key, &req.upload_id,
            req.parts.iter().map(|p| {
                aws_sdk_s3::types::CompletedPart::builder()
                    .part_number(p.part_number)
                    .e_tag(p.e_tag.clone())
                    .build()
            }).collect())
        .await?;

    let response = MultipartCompleteResponse {
        key: req.key.clone(),
        etag,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Check storage backend health
///
/// # Request
/// GET /api/v1/storage/health
///
/// # Response
/// ```json
/// {
///   "status": "healthy",
///   "message": "S3 connection OK"
/// }
/// ```
#[actix_web::get("/storage/health")]
pub async fn storage_health(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    match app_state.storage.health_check().await {
        Ok(_) => {
            let response = StorageHealthResponse {
                status: "healthy".to_string(),
                message: "S3 connection OK".to_string(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            let response = StorageHealthResponse {
                status: "unhealthy".to_string(),
                message: format!("S3 connection failed: {:?}", e),
            };
            Ok(HttpResponse::ServiceUnavailable().json(response))
        }
    }
}

/// Configure S3 operations routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_s3)
        .service(delete_s3)
        .service(presigned_put_url)
        .service(presigned_get_url)
        .service(multipart_init)
        .service(multipart_chunk)
        .service(multipart_complete)
        .service(storage_health);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_s3_request_validation() {
        // DTO validation tests would go here
        // Requires actix-web test framework
    }
}
