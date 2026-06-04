use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use futures::TryStreamExt;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::FileUploadResponse;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const UPLOAD_DIR: &str = "./uploads";

#[post("/api/v1/files/upload")]
pub async fn upload_file(
    pool: web::Data<SqlitePool>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    // Create upload directory if not exists
    fs::create_dir_all(UPLOAD_DIR).map_err(|_| AppError::Internal)?;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map(|f| sanitize_filename(f))
            .ok_or_else(|| AppError::BadRequest("Filename required".to_string()))?;

        let file_id = Uuid::new_v4().to_string();
        let filepath = PathBuf::from(UPLOAD_DIR).join(format!("{}_{}", file_id, filename));

        let mut file = fs::File::create(&filepath).map_err(|_| AppError::Internal)?;
        let mut total_size: i64 = 0;

        while let Ok(Some(chunk)) = field.try_next().await {
            total_size += chunk.len() as i64;

            if total_size > MAX_FILE_SIZE as i64 {
                let _ = fs::remove_file(&filepath);
                return Err(AppError::BadRequest("File size exceeded limit".to_string()));
            }

            std::io::Write::write_all(&mut file, &chunk).map_err(|_| AppError::Internal)?;
        }

        // Store metadata in database
        sqlx::query(
            "INSERT INTO files (id, filename, size, path) VALUES (?, ?, ?, ?)"
        )
        .bind(&file_id)
        .bind(&filename)
        .bind(total_size)
        .bind(filepath.to_str().unwrap())
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        return Ok(HttpResponse::Ok().json(FileUploadResponse {
            id: file_id,
            filename,
            size: total_size,
            uploaded_at: Utc::now().to_rfc3339(),
        }));
    }

    Err(AppError::BadRequest("No file uploaded".to_string()))
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file);
}
