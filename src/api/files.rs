#![allow(dead_code)]
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use bytes::Bytes;
use chrono::Utc;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::cache::metrics::{
    REDIS_CACHE_HITS_TOTAL, REDIS_CACHE_MISSES_TOTAL, REDIS_OPERATIONS_TOTAL,
};
use crate::cache::UploadSessionManager;
use crate::error::{AppError, AppResult};
use crate::models::{
    ChunkedUploadCompleteResponse, ChunkedUploadInitRequest, ChunkedUploadInitResponse,
    ChunkedUploadProgressResponse,
};
use crate::storage::FileMetadata as StorageFileMetadata;
use crate::validation::FileValidator;

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // Wave 2: 100MB

#[post("/files/upload")]
#[allow(clippy::never_loop)]
pub async fn upload_file(
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let max_file_size = app_state.settings.upload.max_file_size_mb * 1024 * 1024;
    let validator = FileValidator::new_default();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        let original_filename = content_disposition
            .get_filename()
            .map(crate::validation::sanitize_filename)
            .ok_or_else(|| AppError::BadRequest("Filename required".to_string()))?;

        let file_id = Uuid::new_v4().to_string();
        let mime_type = mime_guess::from_path(&original_filename)
            .first_or_octet_stream()
            .to_string();

        // Validate filename
        validator.validate_filename(&original_filename)?;

        let mut buffer = Vec::new();
        let mut total_size: i64 = 0;
        let mut hasher = Sha256::new();

        while let Ok(Some(chunk)) = field.try_next().await {
            total_size += chunk.len() as i64;

            if total_size > max_file_size as i64 {
                return Err(AppError::PayloadTooLarge);
            }

            hasher.update(&chunk);
            buffer.extend_from_slice(&chunk);
        }

        // Validate complete file
        validator.validate_file(&original_filename, &mime_type, total_size as usize)?;

        let checksum = format!("{:x}", hasher.finalize());

        let file_metadata = StorageFileMetadata {
            filename: file_id.clone(),
            content_type: mime_type.clone(),
            size: total_size as usize,
            user_id: Uuid::new_v4(),
        };

        app_state
            .storage
            .store(Bytes::from(buffer), file_metadata)
            .await
            .map_err(|_| AppError::Internal)?;

        sqlx::query(
            "INSERT INTO files (id, filename, original_name, size, mime_type, checksum) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&file_id)
        .bind(&original_filename)
        .bind(&original_filename)
        .bind(total_size)
        .bind(&mime_type)
        .bind(&checksum)
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Invalidate list caches after upload (Wave 4 Day 13)
        if let Some(cache) = &app_state.cache {
            for page in 1..=10 {
                for per_page in [10, 20, 50] {
                    let list_key = format!("files:list:{}:{}", page, per_page);
                    let _ = cache.delete(&list_key).await;
                }
            }
            let _ = cache.delete("files:search:*").await.ok();
            REDIS_OPERATIONS_TOTAL
                .with_label_values(&["api_invalidate_on_upload"])
                .inc();
        }

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": file_id,
            "filename": original_filename,
            "size": total_size,
            "mime_type": mime_type,
            "checksum": checksum,
            "created_at": Utc::now().to_rfc3339(),
        })));
    }

    Err(AppError::BadRequest("No file uploaded".to_string()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    id: String,
    filename: String,
    original_name: String,
    size: i64,
    mime_type: String,
    #[serde(rename = "created_at")]
    uploaded_at: String,
    is_public: bool,
}

#[get("/files/{id}")]
pub async fn get_file_metadata(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();
    let cache_key = format!("file:metadata:{}", file_id);

    // Try to get from cache first (Wave 4 Day 13)
    if let Some(cache) = &app_state.cache {
        if let Ok(Some(cached)) = cache.get::<FileMetadataResponse>(&cache_key).await {
            REDIS_CACHE_HITS_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL
                .with_label_values(&["api_metadata_cache_hit"])
                .inc();
            return Ok(HttpResponse::Ok().json(cached));
        } else {
            REDIS_CACHE_MISSES_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL
                .with_label_values(&["api_metadata_cache_miss"])
                .inc();
        }
    }

    // Fetch from database
    let file = sqlx::query_as::<_, (String, String, String, i64, String, String, bool)>(
        "SELECT id, filename, COALESCE(original_name, filename), size, COALESCE(mime_type, ''), uploaded_at, COALESCE(is_public, false) FROM files WHERE id = $1"
    )
    .bind(&file_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let response = FileMetadataResponse {
        id: file.0,
        filename: file.1,
        original_name: file.2,
        size: file.3,
        mime_type: file.4,
        uploaded_at: file.5,
        is_public: file.6,
    };

    // Cache the response (TTL: 1 hour)
    if let Some(cache) = &app_state.cache {
        let ttl = app_state.get_ttl_config().file_metadata();
        let _ = cache.set(&cache_key, &response, Some(ttl)).await;
        REDIS_OPERATIONS_TOTAL
            .with_label_values(&["api_metadata_cache_set"])
            .inc();
    }

    Ok(HttpResponse::Ok().json(response))
}

#[get("/files/{id}/download")]
pub async fn download_file(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();

    let file = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT id, original_name, size FROM files WHERE id = $1",
    )
    .bind(&file_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let file_size = file.2;

    let file_content = app_state
        .storage
        .retrieve(&file_id)
        .await
        .map_err(|_| AppError::NotFound)?;

    // Range リクエスト処理
    if let Some(range_header) = req.headers().get("Range") {
        let range_str = range_header.to_str().unwrap_or("");
        if let Some(range_data) = parse_range_header(range_str, file_size) {
            let partial_content =
                &file_content[range_data.start as usize..=range_data.end as usize];

            return Ok(HttpResponse::PartialContent()
                .content_type(
                    mime_guess::from_path(&file.1)
                        .first_or_octet_stream()
                        .to_string(),
                )
                .insert_header((
                    "Content-Range",
                    format!(
                        "bytes {}-{}/{}",
                        range_data.start, range_data.end, file_size
                    ),
                ))
                .insert_header((
                    "Content-Length",
                    (range_data.end - range_data.start + 1).to_string(),
                ))
                .body(partial_content.to_vec()));
        } else {
            return Err(AppError::BadRequest("Invalid Range header".to_string()));
        }
    }

    Ok(HttpResponse::Ok()
        .content_type(
            mime_guess::from_path(&file.1)
                .first_or_octet_stream()
                .to_string(),
        )
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.1),
        ))
        .insert_header(("Content-Length", file_size.to_string()))
        .body(file_content.to_vec()))
}

#[delete("/files/{id}")]
pub async fn delete_file(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();

    sqlx::query_as::<_, (String,)>("SELECT id FROM files WHERE id = $1")
        .bind(&file_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::NotFound)?;

    app_state
        .storage
        .delete(&file_id)
        .await
        .map_err(|_| AppError::Internal)?;

    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(&file_id)
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Invalidate caches (Wave 4 Day 13)
    if let Some(cache) = &app_state.cache {
        // Invalidate file metadata cache
        let metadata_key = format!("file:metadata:{}", file_id);
        let _ = cache.delete(&metadata_key).await;

        // Invalidate all list caches (all pages)
        // Note: In production, use SCAN command to find all matching keys
        for page in 1..=10 {
            for per_page in [10, 20, 50] {
                let list_key = format!("files:list:{}:{}", page, per_page);
                let _ = cache.delete(&list_key).await;
            }
        }

        // Invalidate search caches
        let _ = cache.delete("files:search:*").await.ok();

        REDIS_OPERATIONS_TOTAL
            .with_label_values(&["api_invalidate_on_delete"])
            .inc();
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "File deleted successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[get("/files")]
pub async fn list_files(
    app_state: web::Data<AppState>,
    query: web::Query<ListFilesQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;
    let cache_key = format!("files:list:{}:{}", page, per_page);

    // Try to get from cache first (Wave 4 Day 13)
    if let Some(cache) = &app_state.cache {
        if let Ok(Some(cached)) = cache.get::<serde_json::Value>(&cache_key).await {
            REDIS_CACHE_HITS_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL
                .with_label_values(&["api_list_cache_hit"])
                .inc();
            return Ok(HttpResponse::Ok().json(cached));
        } else {
            REDIS_CACHE_MISSES_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL
                .with_label_values(&["api_list_cache_miss"])
                .inc();
        }
    }

    // Fetch from database
    let files = sqlx::query_as::<_, (String, String, i64, String, String)>(
        "SELECT id, filename, size, COALESCE(mime_type, ''), uploaded_at FROM files ORDER BY uploaded_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let file_list: Vec<_> = files
        .iter()
        .map(|f| {
            serde_json::json!({
                "id": f.0,
                "filename": f.1,
                "size": f.2,
                "mime_type": f.3,
                "created_at": f.4,
                "url": format!("/api/v1/files/{}/download", f.0),
            })
        })
        .collect();

    let response = serde_json::json!({
        "files": file_list,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total.0,
            "total_pages": (total.0 + (per_page as i64) - 1) / (per_page as i64),
        }
    });

    // Cache the response (TTL: 30 minutes)
    if let Some(cache) = &app_state.cache {
        let ttl = app_state.get_ttl_config().file_list();
        let _ = cache.set(&cache_key, &response, Some(ttl)).await;
        REDIS_OPERATIONS_TOTAL
            .with_label_values(&["api_list_cache_set"])
            .inc();
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug)]
struct RangeData {
    start: i64,
    end: i64,
}

fn parse_range_header(range_str: &str, file_size: i64) -> Option<RangeData> {
    // Parse "bytes=0-499" or "bytes=500-" or "bytes=-500"
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..]; // Skip "bytes="

    if let Some(dash_pos) = range_part.find('-') {
        let start_str = &range_part[..dash_pos];
        let end_str = &range_part[dash_pos + 1..];

        let (start, end) = if start_str.is_empty() {
            // Suffix-byte-range: "-500" (last 500 bytes)
            if let Ok(suffix_len) = end_str.parse::<i64>() {
                let start = (file_size - suffix_len).max(0);
                (start, file_size - 1)
            } else {
                return None;
            }
        } else if end_str.is_empty() {
            // Prefix-byte-range: "500-" (from 500 to end)
            if let Ok(start) = start_str.parse::<i64>() {
                if start >= file_size {
                    return None;
                }
                (start, file_size - 1)
            } else {
                return None;
            }
        } else {
            // Absolute-byte-range: "0-499"
            match (start_str.parse::<i64>(), end_str.parse::<i64>()) {
                (Ok(start), Ok(end)) => {
                    if start > end || start >= file_size {
                        return None;
                    }
                    let end = end.min(file_size - 1);
                    (start, end)
                }
                _ => return None,
            }
        };

        return Some(RangeData { start, end });
    }

    None
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[get("/files/stats")]
pub async fn file_stats(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT COUNT(*), COALESCE(SUM(size), 0), COALESCE(AVG(size), 0), COUNT(DISTINCT mime_type) FROM files"
    )
    .fetch_one(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_files": row.0,
        "total_size_bytes": row.1,
        "average_size_bytes": row.2,
        "unique_mime_types": row.3,
    })))
}

// ============================================================
// Wave 2B: Chunked Upload API
// ============================================================

#[post("/files/upload/init")]
pub async fn chunked_upload_init(
    app_state: web::Data<AppState>,
    req: web::Json<ChunkedUploadInitRequest>,
) -> AppResult<HttpResponse> {
    let session_id = Uuid::new_v4().to_string();
    let chunk_size = req.chunk_size.unwrap_or(5 * 1024 * 1024); // Default 5MB
    let total_chunks = (req.file_size + chunk_size - 1) / chunk_size;

    // Create upload session in database
    sqlx::query(
        "INSERT INTO upload_sessions (id, file_id, user_id, total_size, chunk_size, status, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7)"
    )
    .bind(&session_id)
    .bind(Uuid::new_v4().to_string())
    .bind(Uuid::new_v4().to_string())
    .bind(req.file_size)
    .bind(chunk_size)
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(crate::models::ApiResponse::success(
        ChunkedUploadInitResponse {
            session_id,
            chunk_size,
            total_chunks,
        },
    )))
}

#[derive(Debug, Deserialize)]
pub struct ChunkedUploadChunkRequest {
    pub session_id: String,
    pub chunk_index: i64,
}

#[post("/files/upload/chunk")]
pub async fn chunked_upload_chunk(
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let mut session_id = String::new();
    let mut chunk_index: i32 = 0;
    let mut chunk_data = Vec::new();
    let mut uploaded_size: i64 = 0;
    let mut total_size: i64 = 0;

    // Parse multipart form data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.name().to_string();

        match name.as_str() {
            "session_id" => {
                let data = field
                    .try_next()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read session_id".to_string()))?
                    .ok_or_else(|| AppError::BadRequest("Empty session_id".to_string()))?;
                session_id = String::from_utf8(data.to_vec())
                    .map_err(|_| AppError::BadRequest("Invalid session_id encoding".to_string()))?;
            }
            "chunk_index" => {
                let data = field
                    .try_next()
                    .await
                    .map_err(|_| AppError::BadRequest("Failed to read chunk_index".to_string()))?
                    .ok_or_else(|| AppError::BadRequest("Empty chunk_index".to_string()))?;
                let index_str = String::from_utf8(data.to_vec()).map_err(|_| {
                    AppError::BadRequest("Invalid chunk_index encoding".to_string())
                })?;
                chunk_index = index_str
                    .parse()
                    .map_err(|_| AppError::BadRequest("Invalid chunk_index format".to_string()))?;
            }
            "chunk" => {
                while let Ok(Some(data)) = field.try_next().await {
                    chunk_data.extend_from_slice(&data);
                }
            }
            _ => {}
        }
    }

    if session_id.is_empty() {
        return Err(AppError::BadRequest("session_id required".to_string()));
    }

    // Fetch session from database
    let session = sqlx::query_as::<_, (String, i64, i64, String)>(
        "SELECT id, total_size, chunk_size, status FROM upload_sessions WHERE id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    total_size = session.1;
    let chunk_size = session.2;
    uploaded_size = (chunk_index as i64) * chunk_size;

    // Update session: increment uploaded_size
    let new_uploaded_size =
        ((chunk_index as i64) + 1) * chunk_size.min(total_size - (chunk_index as i64) * chunk_size);

    sqlx::query("UPDATE upload_sessions SET uploaded_size = $1, updated_at = $2 WHERE id = $3")
        .bind(new_uploaded_size)
        .bind(Utc::now().to_rfc3339())
        .bind(&session_id)
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update Redis cache if available (optional performance optimization)
    if let Some(redis) = app_state.get_cache() {
        let upload_session_mgr = UploadSessionManager::new(redis.clone());
        // Try to update cache, but don't fail if Redis is unavailable
        let _ = upload_session_mgr
            .update_progress(&session_id, new_uploaded_size, chunk_index)
            .await;
    }

    let progress_percent = (new_uploaded_size as f32 / total_size as f32) * 100.0;

    Ok(HttpResponse::Ok().json(crate::models::ApiResponse::success(
        ChunkedUploadProgressResponse {
            session_id: session_id.clone(),
            uploaded_size: new_uploaded_size,
            total_size,
            progress_percent,
            status: "uploading".to_string(),
        },
    )))
}

#[post("/files/upload/complete/{session_id}")]
pub async fn chunked_upload_complete(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();

    // Fetch session
    let session = sqlx::query_as::<_, (String, String, i64, i64, String)>(
        "SELECT id, file_id, total_size, uploaded_size, status FROM upload_sessions WHERE id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let (_, file_id, total_size, uploaded_size, _) = session;

    // Verify upload completion
    if uploaded_size < total_size {
        return Err(AppError::BadRequest(format!(
            "Upload incomplete: {} / {} bytes",
            uploaded_size, total_size
        )));
    }

    // Mark session as completed
    sqlx::query("UPDATE upload_sessions SET status = 'completed', updated_at = $1 WHERE id = $2")
        .bind(Utc::now().to_rfc3339())
        .bind(&session_id)
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create file record if it doesn't exist (for chunked upload)
    let file_name = format!("chunked_upload_{}", file_id);
    let mime_type = "application/octet-stream";
    let checksum = req
        .get("checksum")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    sqlx::query(
        "INSERT OR IGNORE INTO files (id, filename, original_name, size, mime_type, checksum, uploaded_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&file_id)
    .bind(&file_name)
    .bind(&file_name)
    .bind(total_size)
    .bind(mime_type)
    .bind(&checksum)
    .bind(Utc::now().to_rfc3339())
    .execute(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Mark session as completed in Redis cache if available
    if let Some(redis) = app_state.get_cache() {
        let upload_session_mgr = UploadSessionManager::new(redis.clone());
        let _ = upload_session_mgr
            .mark_completed(&session_id, &file_id)
            .await;
    }

    Ok(HttpResponse::Ok().json(crate::models::ApiResponse::success(
        ChunkedUploadCompleteResponse {
            file_id: file_id.clone(),
            filename: file_name,
            size: total_size,
            mime_type: mime_type.to_string(),
            checksum,
            created_at: Utc::now().to_rfc3339(),
        },
    )))
}

#[get("/files/upload/progress/{session_id}")]
pub async fn chunked_upload_progress(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();

    // Try Redis cache first if available
    if let Some(redis) = app_state.get_cache() {
        let upload_session_mgr = UploadSessionManager::new(redis.clone());
        if let Ok(Some(cached_session)) = upload_session_mgr.get_session(&session_id).await {
            return Ok(HttpResponse::Ok().json(crate::models::ApiResponse::success(
                ChunkedUploadProgressResponse {
                    session_id,
                    uploaded_size: cached_session.uploaded_size,
                    total_size: cached_session.total_size,
                    progress_percent: cached_session.progress_percent(),
                    status: cached_session.status,
                },
            )));
        }
    }

    // Fallback to SQLite
    let session = sqlx::query_as::<_, (i64, i64, String)>(
        "SELECT total_size, uploaded_size, status FROM upload_sessions WHERE id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let (total_size, uploaded_size, status) = session;
    let progress_percent = (uploaded_size as f32 / total_size as f32) * 100.0;

    Ok(HttpResponse::Ok().json(crate::models::ApiResponse::success(
        ChunkedUploadProgressResponse {
            session_id,
            uploaded_size,
            total_size,
            progress_percent,
            status,
        },
    )))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file_stats) // /files/stats (specific path, register early)
        .service(chunked_upload_init) // /files/upload/init (longer path)
        .service(chunked_upload_chunk) // /files/upload/chunk
        .service(chunked_upload_complete) // /files/upload/complete/{session_id}
        .service(chunked_upload_progress) // /files/upload/progress/{session_id}
        .service(upload_file) // /files/upload (general multipart)
        .service(get_file_metadata) // /files/{id}
        .service(download_file) // /files/{id}/download
        .service(delete_file) // /files/{id}
        .service(list_files); // /files (generic list)
}
