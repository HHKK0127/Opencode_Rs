use actix_multipart::Multipart;
use actix_web::{post, get, delete, web, HttpResponse};
use chrono::Utc;
use futures::TryStreamExt;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use bytes::Bytes;

use crate::error::{AppError, AppResult};
use crate::app_state::AppState;
use crate::storage::{FileMetadata as StorageFileMetadata};
use crate::validation::FileValidator;

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
            filename: original_filename.clone(),
            content_type: mime_type.clone(),
            size: total_size as usize,
            user_id: Uuid::new_v4(),
        };

        app_state.storage
            .store(Bytes::from(buffer), file_metadata)
            .await
            .map_err(|_| AppError::Internal)?;

        sqlx::query(
            "INSERT INTO files (id, filename, original_name, size, mime_type, checksum, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&file_id)
        .bind(&original_filename)
        .bind(&original_filename)
        .bind(total_size)
        .bind(&mime_type)
        .bind(&checksum)
        .bind(Utc::now().to_rfc3339())
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

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

#[derive(Debug, Serialize)]
pub struct FileMetadataResponse {
    id: String,
    filename: String,
    original_name: String,
    size: i64,
    mime_type: String,
    created_at: String,
    is_public: bool,
}

#[get("/files/{id}")]
pub async fn get_file_metadata(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();

    let file = sqlx::query_as::<_, (String, String, String, i64, String, String, bool)>(
        "SELECT id, filename, original_name, size, mime_type, created_at, is_public FROM files WHERE id = ?"
    )
    .bind(&file_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    Ok(HttpResponse::Ok().json(FileMetadataResponse {
        id: file.0,
        filename: file.1,
        original_name: file.2,
        size: file.3,
        mime_type: file.4,
        created_at: file.5,
        is_public: file.6,
    }))
}

#[get("/files/{id}/download")]
pub async fn download_file(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let file_id = path.into_inner();

    let file = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT id, original_name, size FROM files WHERE id = ?"
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
        "SELECT id FROM files WHERE id = ?"
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

    sqlx::query("DELETE FROM files WHERE id = ?")
        .bind(&file_id)
        .execute(&app_state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

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

    let files = sqlx::query_as::<_, (String, String, i64, String, String)>(
        "SELECT id, filename, size, mime_type, created_at FROM files ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(per_page)
    .bind(offset)
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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "files": file_list,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total.0,
            "total_pages": (total.0 + (per_page as i64) - 1) / (per_page as i64),
        }
    })))
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file)
        .service(get_file_metadata)
        .service(download_file)
        .service(delete_file)
        .service(list_files);
}
