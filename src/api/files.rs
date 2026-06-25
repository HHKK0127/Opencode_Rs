use actix_multipart::Multipart;
use actix_web::{post, get, delete, web, HttpResponse};
use chrono::Utc;
use futures::TryStreamExt;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::app_state::AppState;
use crate::storage::{FileMetadata as StorageFileMetadata};
use crate::validation::FileValidator;
use crate::cache::metrics::{REDIS_OPERATIONS_TOTAL, REDIS_CACHE_HITS_TOTAL, REDIS_CACHE_MISSES_TOTAL};

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // Wave 2: 100MB

#[post("/files/upload")]
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
            .map(|f| crate::validation::sanitize_filename(f))
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

        app_state.storage
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
            REDIS_OPERATIONS_TOTAL.with_label_values(&["api_invalidate_on_upload"]).inc();
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
            REDIS_OPERATIONS_TOTAL.with_label_values(&["api_metadata_cache_hit"]).inc();
            return Ok(HttpResponse::Ok().json(cached));
        } else {
            REDIS_CACHE_MISSES_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL.with_label_values(&["api_metadata_cache_miss"]).inc();
        }
    }

    // Fetch from database
    let file = sqlx::query_as::<_, (String, String, String, i64, String, String, bool)>(
        "SELECT id, filename, COALESCE(original_name, filename), size, COALESCE(mime_type, ''), uploaded_at::text, COALESCE(is_public, false) FROM files WHERE id = $1"
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
        REDIS_OPERATIONS_TOTAL.with_label_values(&["api_metadata_cache_set"]).inc();
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
        "SELECT id, original_name, size FROM files WHERE id = $1"
    )
    .bind(&file_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let file_size = file.2;

    let file_content = app_state.storage
        .retrieve(&file_id)
        .await
        .map_err(|_| AppError::NotFound)?;

    // Range リクエスト処理
    if let Some(range_header) = req.headers().get("Range") {
        let range_str = range_header.to_str().unwrap_or("");
        if let Some(range_data) = parse_range_header(range_str, file_size) {
            let partial_content = &file_content[range_data.start as usize..=range_data.end as usize];

            return Ok(HttpResponse::PartialContent()
                .content_type(mime_guess::from_path(&file.1).first_or_octet_stream().to_string())
                .insert_header((
                    "Content-Range",
                    format!("bytes {}-{}/{}", range_data.start, range_data.end, file_size),
                ))
                .insert_header(("Content-Length", (range_data.end - range_data.start + 1).to_string()))
                .body(partial_content.to_vec()));
        } else {
            return Err(AppError::BadRequest("Invalid Range header".to_string()));
        }
    }

    Ok(HttpResponse::Ok()
        .content_type(mime_guess::from_path(&file.1).first_or_octet_stream().to_string())
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file.1)))
        .insert_header(("Content-Length", file_size.to_string()))
        .body(file_content.to_vec()))
}

#[delete("/files/{id}")]
pub async fn delete_file(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();

    sqlx::query_as::<_, (String,)>(
        "SELECT id FROM files WHERE id = $1"
    )
    .bind(&file_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    app_state.storage
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

        REDIS_OPERATIONS_TOTAL.with_label_values(&["api_invalidate_on_delete"]).inc();
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
            REDIS_OPERATIONS_TOTAL.with_label_values(&["api_list_cache_hit"]).inc();
            return Ok(HttpResponse::Ok().json(cached));
        } else {
            REDIS_CACHE_MISSES_TOTAL.inc();
            REDIS_OPERATIONS_TOTAL.with_label_values(&["api_list_cache_miss"]).inc();
        }
    }

    // Fetch from database
    let files = sqlx::query_as::<_, (String, String, i64, String, String)>(
        "SELECT id, filename, size, COALESCE(mime_type, ''), uploaded_at::text FROM files ORDER BY uploaded_at DESC LIMIT $1 OFFSET $2"
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
        .map(|f| serde_json::json!({
            "id": f.0,
            "filename": f.1,
            "size": f.2,
            "mime_type": f.3,
            "created_at": f.4,
            "url": format!("/api/v1/files/{}/download", f.0),
        }))
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
        REDIS_OPERATIONS_TOTAL.with_label_values(&["api_list_cache_set"]).inc();
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
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[get("/files/stats")]
pub async fn file_stats(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT COUNT(*), COALESCE(SUM(size), 0)::BIGINT, COALESCE(AVG(size), 0)::BIGINT, COUNT(DISTINCT mime_type) FROM files"
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file)
        .service(file_stats)        // must be before get_file_metadata (/files/{id})
        .service(get_file_metadata)
        .service(download_file)
        .service(delete_file)
        .service(list_files);
}
