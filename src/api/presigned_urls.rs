use actix_web::{post, web, HttpResponse};
use tracing::{error, info};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::app_state::AppState;
use crate::models::{PresignedPutRequest, PresignedGetRequest, PresignedUrlResponse};

/// Generate presigned PUT URL for client-side direct upload
///
/// POST /api/v1/files/s3/presigned-put
///
/// Request body:
/// {
///   "filename": "document.pdf",
///   "content_type": "application/pdf",  // optional
///   "expires_in_seconds": 300           // optional, default 300
/// }
#[post("/files/s3/presigned-put")]
pub async fn get_presigned_put_url(
    app_state: web::Data<AppState>,
    req: web::Json<PresignedPutRequest>,
) -> AppResult<HttpResponse> {
    info!(
        "Generating presigned PUT URL for: {}",
        req.filename
    );

    // Validate filename
    if req.filename.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Filename cannot be empty".to_string(),
        ));
    }

    // Use provided TTL or default to 300 seconds (5 minutes)
    let expires_in_seconds = req.expires_in_seconds.unwrap_or(300);

    // Validate expiry time (max 24 hours)
    if expires_in_seconds > 86400 {
        return Err(AppError::BadRequest(
            "Expiry time cannot exceed 24 hours (86400 seconds)".to_string(),
        ));
    }

    if expires_in_seconds < 1 {
        return Err(AppError::BadRequest(
            "Expiry time must be at least 1 second".to_string(),
        ));
    }

    // Convert expiry seconds to Duration
    let expires_in = Duration::from_secs(expires_in_seconds);

    // Generate the presigned URL using S3 client
    match app_state.s3_client
        .generate_presigned_put_url(
            &req.filename,
            expires_in,
            req.content_type.as_deref(),
        )
        .await
    {
        Ok(presigned_url) => {
            info!(
                "Presigned PUT URL generated successfully for: {} (TTL: {}s)",
                req.filename, expires_in_seconds
            );

            let response = PresignedUrlResponse {
                presigned_url,
                expires_in_seconds,
                bucket: app_state.settings.storage.s3.bucket.clone(),
                key: req.filename.clone(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to generate presigned PUT URL: {:?}", e);
            Err(e)
        }
    }
}

/// Generate presigned GET URL for client-side direct download
///
/// POST /api/v1/files/s3/presigned-get
///
/// Request body:
/// {
///   "file_id": "uploads/2026/05/28/file-abc123.pdf",
///   "expires_in_seconds": 3600  // optional, default 3600
/// }
#[post("/files/s3/presigned-get")]
pub async fn get_presigned_get_url(
    app_state: web::Data<AppState>,
    req: web::Json<PresignedGetRequest>,
) -> AppResult<HttpResponse> {
    info!(
        "Generating presigned GET URL for file_id: {}",
        req.file_id
    );

    // Validate file_id
    if req.file_id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "File ID cannot be empty".to_string(),
        ));
    }

    // Use provided TTL or default to 3600 seconds (1 hour)
    let expires_in_seconds = req.expires_in_seconds.unwrap_or(3600);

    // Validate expiry time (max 24 hours)
    if expires_in_seconds > 86400 {
        return Err(AppError::BadRequest(
            "Expiry time cannot exceed 24 hours (86400 seconds)".to_string(),
        ));
    }

    if expires_in_seconds < 1 {
        return Err(AppError::BadRequest(
            "Expiry time must be at least 1 second".to_string(),
        ));
    }

    // Convert expiry seconds to Duration
    let expires_in = Duration::from_secs(expires_in_seconds);

    // Generate the presigned URL using S3 client
    match app_state.s3_client
        .generate_presigned_get_url(&req.file_id, expires_in)
        .await
    {
        Ok(presigned_url) => {
            info!(
                "Presigned GET URL generated successfully for file: {} (TTL: {}s)",
                req.file_id, expires_in_seconds
            );

            let response = PresignedUrlResponse {
                presigned_url,
                expires_in_seconds,
                bucket: app_state.settings.storage.s3.bucket.clone(),
                key: req.file_id.clone(),
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to generate presigned GET URL: {:?}", e);
            Err(e)
        }
    }
}

/// Configure presigned URL routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_presigned_put_url)
        .service(get_presigned_get_url);
}
