use actix_web::{post, web, HttpResponse};
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::file_metadata::{
    FileMetadataRegisterRequest,
    FileMetadataResponse,
    S3UploadCompleteRequest,
};

type AppResult<T> = Result<T, AppError>;

#[post("/files/register")]
pub async fn register_file_metadata(
    req: web::Json<FileMetadataRegisterRequest>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    // バリデーション
    if req.filename.is_empty() || req.filename.len() > 255 {
        return Err(AppError::BadRequest("Invalid filename length".to_string()));
    }
    if req.s3_path.is_empty() {
        return Err(AppError::BadRequest("S3 path is required".to_string()));
    }
    if req.s3_etag.is_empty() {
        return Err(AppError::BadRequest("S3 ETag is required".to_string()));
    }
    if req.size <= 0 {
        return Err(AppError::BadRequest("Size must be positive".to_string()));
    }

    info!("Registering file metadata: {}", req.filename);

    // S3オブジェクト存在確認（HeadObject）
    let s3_key = extract_s3_key(&req.s3_path)?;
    
    let head_result = state.s3_client
        .head_object(&s3_key)
        .await
        .map_err(|e| {
            error!("S3 HeadObject failed: {:?}", e);
            AppError::BadRequest("S3 object not found or inaccessible".to_string())
        })?;

    // サイズ整合性チェック
    let s3_size = head_result.content_length.unwrap_or(0) as i64;
    if s3_size != req.size {
        warn!("Size mismatch: expected {}, actual {}", req.size, s3_size);
        return Err(AppError::BadRequest(
            format!("Size mismatch: expected {}, actual {}", req.size, s3_size)
        ));
    }

    // ETag整合性チェック
    let s3_etag = head_result.e_tag.unwrap_or_default().trim_matches('"').to_string();
    if s3_etag != req.s3_etag {
        warn!("ETag mismatch: expected {}, actual {}", req.s3_etag, s3_etag);
        return Err(AppError::BadRequest("ETag mismatch".to_string()));
    }

    // DBにメタデータ登録
    let file_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO files (
            id, filename, size, mime_type,
            s3_path, s3_etag, s3_version_id, storage_type,
            metadata, created_at, uploaded_at, path
        ) VALUES (?, ?, ?, ?, ?, ?, ?, 's3', ?, ?, ?, ?)
        "#,
    )
    .bind(&file_id)
    .bind(&req.filename)
    .bind(req.size)
    .bind(&req.mime_type)
    .bind(&req.s3_path)
    .bind(&s3_etag)
    .bind(&req.s3_version_id)
    .bind(req.metadata.as_ref().map(|m| m.to_string()))
    .bind(&now_str)
    .bind(&now_str)
    .bind::<Option<String>>(None) // path is NULL for S3
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Database insert failed: {}", e);
        AppError::Database(e.to_string())
    })?;

    info!("File metadata registered: {} (S3: {})", file_id, req.s3_path);

    // Presigned GET URL生成（ダウンロード用）
    let download_url = state.s3_client
        .generate_presigned_get_url(&s3_key, std::time::Duration::from_secs(3600))
        .await?;

    let response = FileMetadataResponse {
        id: file_id,
        filename: req.filename.clone(),
        original_name: req.filename.clone(),
        size: req.size,
        mime_type: req.mime_type.clone(),
        s3_path: req.s3_path.clone(),
        s3_etag: s3_etag,
        s3_version_id: req.s3_version_id.clone(),
        storage_type: "s3".to_string(),
        download_url,
        created_at: now.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/files/s3/complete")]
pub async fn complete_s3_upload(
    req: web::Json<S3UploadCompleteRequest>,
    state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    info!("Completing S3 upload: {}", req.filename);

    // S3オブジェクト存在確認
    let s3_key = extract_s3_key(&req.s3_path)?;
    
    let head_result = state.s3_client
        .head_object(&s3_key)
        .await
        .map_err(|e| {
            error!("S3 HeadObject failed for completion: {:?}", e);
            AppError::BadRequest("S3 upload not found".to_string())
        })?;

    // 自動メタデータ補完
    let actual_size = head_result.content_length.unwrap_or(0) as i64;
    let actual_etag = head_result.e_tag.unwrap_or_default().trim_matches('"').to_string();
    let actual_mime = head_result.content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    // DB登録
    let file_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO files (
            id, filename, size, mime_type,
            s3_path, s3_etag, storage_type, created_at, uploaded_at, path
        ) VALUES (?, ?, ?, ?, ?, ?, 's3', ?, ?, ?)
        "#,
    )
    .bind(&file_id)
    .bind(&req.filename)
    .bind(actual_size)
    .bind(&actual_mime)
    .bind(&req.s3_path)
    .bind(&actual_etag)
    .bind(&now_str)
    .bind(&now_str)
    .bind::<Option<String>>(None)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Database insert failed: {}", e);
        AppError::Database(e.to_string())
    })?;

    // Presigned URL生成
    let download_url = state.s3_client
        .generate_presigned_get_url(&s3_key, std::time::Duration::from_secs(3600))
        .await?;

    let response = FileMetadataResponse {
        id: file_id,
        filename: req.filename.clone(),
        original_name: req.filename.clone(),
        size: actual_size,
        mime_type: Some(actual_mime),
        s3_path: req.s3_path.clone(),
        s3_etag: actual_etag,
        s3_version_id: None,
        storage_type: "s3".to_string(),
        download_url,
        created_at: now.to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

// S3パスからキーを抽出（"s3://bucket/key" → "key"）
fn extract_s3_key(s3_path: &str) -> AppResult<String> {
    if !s3_path.starts_with("s3://") {
        return Err(AppError::BadRequest("Invalid S3 path format".to_string()));
    }

    let parts: Vec<&str> = s3_path.splitn(3, '/').collect();
    if parts.len() < 3 {
        return Err(AppError::BadRequest("Invalid S3 path: missing key".to_string()));
    }

    Ok(parts[2].to_string())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register_file_metadata)
        .service(complete_s3_upload);
}
