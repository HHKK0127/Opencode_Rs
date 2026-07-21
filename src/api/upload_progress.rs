use actix_web::{get, web, HttpResponse};
use serde::Serialize;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct UploadProgress {
    session_id: String,
    file_id: String,
    total_size: i64,
    uploaded_size: i64,
    progress_percent: f64,
    status: String,
    remaining_bytes: i64,
}

#[get("/files/upload/progress/{session_id}")]
pub async fn get_upload_progress(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();

    let session = sqlx::query_as::<_, (String, String, i64, i64, String)>(
        "SELECT id, file_id, total_size, uploaded_size, status FROM upload_sessions WHERE id = $1",
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or(AppError::NotFound)?;

    let (session_id, file_id, total_size, uploaded_size, status) = session;

    let progress_percent = if total_size > 0 {
        (uploaded_size as f64 / total_size as f64) * 100.0
    } else {
        0.0
    };

    let remaining_bytes = total_size - uploaded_size;

    Ok(HttpResponse::Ok().json(UploadProgress {
        session_id,
        file_id,
        total_size,
        uploaded_size,
        progress_percent,
        status,
        remaining_bytes,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_upload_progress);
}
