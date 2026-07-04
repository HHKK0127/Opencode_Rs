use actix_web::{post, get, web, HttpResponse};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::io::Write;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use futures::TryStreamExt;
use tempfile::NamedTempFile;
use tracing::{info, warn, error};
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::app_state::AppState;

#[derive(Debug, Deserialize)]
pub struct InitChunkedUploadRequest {
    filename: String,
    total_size: i64,
    chunk_size: usize,
}

#[derive(Debug, Serialize)]
pub struct InitChunkedUploadResponse {
    session_id: String,
    chunk_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct UploadChunkRequest {
    session_id: String,
    chunk_index: u32,
}

#[derive(Debug, Serialize)]
pub struct UploadChunkResponse {
    session_id: String,
    chunk_index: u32,
    uploaded_size: i64,
    status: String,
}

#[derive(Debug, Serialize)]
pub struct ChunkUploadSession {
    session_id: String,
    file_id: String,
    total_size: i64,
    uploaded_size: i64,
    chunk_size: usize,
    status: String,
    created_at: String,
}

#[post("/files/upload/init")]
pub async fn init_chunked_upload(
    app_state: web::Data<AppState>,
    req: web::Json<InitChunkedUploadRequest>,
) -> AppResult<HttpResponse> {
    // 入力検証
    if req.filename.is_empty() {
        return Err(AppError::BadRequest("Filename cannot be empty".to_string()));
    }
    if req.total_size <= 0 {
        return Err(AppError::BadRequest("Invalid file size".to_string()));
    }
    if req.chunk_size == 0 || req.chunk_size > 100 * 1024 * 1024 {
        return Err(AppError::BadRequest("Invalid chunk size (max 100MB)".to_string()));
    }

    let max_file_size = app_state.settings.upload.max_file_size_mb * 1024 * 1024;

    if req.total_size > max_file_size as i64 {
        return Err(AppError::BadRequest(
            format!("File size {} exceeds limit of {}MB",
                req.total_size,
                max_file_size / 1024 / 1024)
        ));
    }

    let session_id = Uuid::new_v4().to_string();
    let file_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO upload_sessions (id, file_id, total_size, uploaded_size, chunk_size, status)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(&session_id)
    .bind(&file_id)
    .bind(req.total_size)
    .bind(0i64)
    .bind(req.chunk_size as i64)
    .bind("uploading")
    .execute(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Chunked upload initialized: session={}, file={}", session_id, file_id);

    Ok(HttpResponse::Ok().json(InitChunkedUploadResponse {
        session_id,
        chunk_size: req.chunk_size,
    }))
}

#[post("/files/upload/chunk/{session_id}")]
pub async fn upload_chunk(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();
    let upload_dir = &app_state.settings.upload.directory;

    // セッション検証
    let session = sqlx::query_as::<_, (String, String, i64, i64, i64)>(
        "SELECT id, file_id, total_size, uploaded_size, chunk_size FROM upload_sessions WHERE id = $1 AND status = 'uploading'"
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound)?;

    let (_, file_id, total_size, uploaded_size, chunk_size) = session;

    // 一時ディレクトリ作成
    let temp_dir = PathBuf::from(upload_dir).join("temp").join(&session_id);
    tokio::fs::create_dir_all(&temp_dir).await.map_err(|e| {
        error!("Failed to create temp directory: {}", e);
        AppError::Internal
    })?;

    // チャンクデータ読み込み
    let mut chunk_index = 0u32;
    let mut chunk_data = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let field_name = field.name().to_string();

        if field_name == "chunk_index" {
            let mut text = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                text.push_str(&String::from_utf8_lossy(&chunk));
            }
            chunk_index = text.trim().parse::<u32>().unwrap_or(0);
        } else if field_name == "chunk" {
            while let Ok(Some(chunk)) = field.try_next().await {
                chunk_data.extend_from_slice(&chunk);
            }
        }
    }

    // サイズ検証
    if chunk_data.len() > chunk_size as usize * 2 {
        return Err(AppError::BadRequest("Chunk size exceeds limit".to_string()));
    }

    let new_uploaded_size = uploaded_size + chunk_data.len() as i64;
    if new_uploaded_size > total_size {
        return Err(AppError::BadRequest("Upload exceeds total size".to_string()));
    }

    // === RAIIパターンによる一時ファイル管理 ===
    let chunk_path = temp_dir.join(format!("chunk-{}", chunk_index));
    
    // NamedTempFileを使用（Drop時に自動削除）
    let temp_file = match NamedTempFile::new_in(&temp_dir) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create temp file: {}", e);
            return Err(AppError::Internal);
        }
    };

    // データ書き込み
    {
        let mut file = temp_file.as_file();
        if let Err(e) = file.write_all(&chunk_data) {
            error!("Failed to write chunk: {}", e);
            return Err(AppError::Internal);
        }
        if let Err(e) = file.sync_all() {
            error!("Failed to sync chunk file: {}", e);
            return Err(AppError::Internal);
        }
    }

    // DB更新（トランザクション）
    match sqlx::query(
        "UPDATE upload_sessions SET uploaded_size = $1 WHERE id = $2"
    )
    .bind(new_uploaded_size)
    .bind(&session_id)
    .execute(&app_state.db)
    .await
    {
        Ok(_) => {
            // 成功：一時ファイルを永続化
            if let Err(e) = temp_file.persist(&chunk_path) {
                error!("Failed to persist temp file: {}", e);
                return Err(AppError::Internal);
            }
            
            info!("Chunk {} uploaded for session {}", chunk_index, session_id);
            
            let status = if new_uploaded_size == total_size {
                "completed"
            } else {
                "uploading"
            };

            Ok(HttpResponse::Ok().json(UploadChunkResponse {
                session_id,
                chunk_index,
                uploaded_size: new_uploaded_size,
                status: status.to_string(),
            }))
        }
        Err(e) => {
            // 失敗：temp_fileがDropで自動削除される
            error!("DB update failed, temp file will be cleaned up: {}", e);
            Err(AppError::Database(e.to_string()))
        }
    }
}

#[post("/files/upload/complete/{session_id}")]
pub async fn complete_chunked_upload(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();
    let upload_dir = &app_state.settings.upload.directory;

    let session = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT id, file_id, total_size, uploaded_size FROM upload_sessions WHERE id = $1 AND status = 'uploading'"
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound)?;

    let (session_id, file_id, total_size, uploaded_size) = session;

    if uploaded_size != total_size {
        return Err(AppError::BadRequest(
            format!("Upload incomplete: {} of {} bytes", uploaded_size, total_size)
        ));
    }

    let temp_dir = PathBuf::from(upload_dir).join("temp").join(&session_id);
    let today = chrono::Local::now().format("%Y/%m/%d").to_string();
    let final_dir = PathBuf::from(upload_dir).join(&today);
    
    tokio::fs::create_dir_all(&final_dir).await.map_err(|e| {
        error!("Failed to create final directory: {}", e);
        AppError::Internal
    })?;

    let final_filename = format!("{}-chunked", file_id);
    let final_path = final_dir.join(&final_filename);
    let mut final_file = tokio::fs::File::create(&final_path).await.map_err(|e| {
        error!("Failed to create final file: {}", e);
        AppError::Internal
    })?;
    let mut hasher = Sha256::new();

    let mut chunk_index = 0u32;
    loop {
        let chunk_path = temp_dir.join(format!("chunk-{}", chunk_index));
        
        match tokio::fs::read(&chunk_path).await {
            Ok(chunk_data) => {
                hasher.update(&chunk_data);
                if let Err(e) = final_file.write_all(&chunk_data).await {
                    error!("Failed to write to final file: {}", e);
                    return Err(AppError::Internal);
                }

                // 一時ファイル削除
                if let Err(e) = tokio::fs::remove_file(&chunk_path).await {
                    warn!("Failed to remove chunk file: {}", e);
                }
                chunk_index += 1;
            }
            Err(_) => break, // チャンク終了
        }
    }

    // 一時ディレクトリ削除
    if let Err(e) = tokio::fs::remove_dir_all(&temp_dir).await {
        warn!("Failed to remove temp directory: {}", e);
    }

    let checksum = format!("{:x}", hasher.finalize());

    let mime_type = mime_guess::from_path(&file_id)
        .first_or_octet_stream()
        .to_string();

    sqlx::query(
        "INSERT INTO files (id, filename, original_name, size, mime_type, path, checksum)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(&file_id)
    .bind(&final_filename)
    .bind(&final_filename)
    .bind(uploaded_size)
    .bind(&mime_type)
    .bind(final_path.to_str().unwrap())
    .bind(&checksum)
    .execute(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        "UPDATE upload_sessions SET status = $1 WHERE id = $2"
    )
    .bind("completed")
    .bind(&session_id)
    .execute(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Upload completed: file_id={}, size={}", file_id, uploaded_size);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "file_id": file_id,
        "size": uploaded_size,
        "checksum": checksum,
        "path": format!("/uploads/{}/{}", today, final_filename),
    })))
}

#[get("/files/upload/sessions/{session_id}")]
pub async fn get_upload_session(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let session_id = path.into_inner();

    let session = sqlx::query_as::<_, (String, String, i64, i64, i64, String, String)>(
        "SELECT id, file_id, total_size, uploaded_size, chunk_size, status, created_at::text FROM upload_sessions WHERE id = $1"
    )
    .bind(&session_id)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound)?;

    let (id, file_id, total_size, uploaded_size, chunk_size, status, created_at) = session;

    Ok(HttpResponse::Ok().json(ChunkUploadSession {
        session_id: id,
        file_id,
        total_size,
        uploaded_size,
        chunk_size: chunk_size as usize,
        status,
        created_at,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(init_chunked_upload)
        .service(upload_chunk)
        .service(complete_chunked_upload)
        .service(get_upload_session);
}
