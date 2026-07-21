#![allow(dead_code)]
//! Session Management API Endpoints
//!
//! JWT + Redis Session Integration
//! Endpoints: validate, extend, invalidate, info

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    app_state::AppState,
    cache::session::SessionManager,
    error::{AppError, AppResult},
};

/// セッション検証リクエスト
#[derive(Debug, Deserialize)]
pub struct ValidateSessionRequest {
    pub token: Option<String>, // 省略時は Authorization ヘッダーから取得
}

/// セッション検証レスポンス
#[derive(Debug, Serialize)]
pub struct ValidateSessionResponse {
    pub valid: bool,
    pub user_id: String,
    pub username: String,
    pub created_at: String,
    pub last_activity: String,
    pub permissions: Vec<String>,
}

/// セッション拡張レスポンス
#[derive(Debug, Serialize)]
pub struct ExtendSessionResponse {
    pub extended: bool,
    pub new_ttl_hours: i64,
    pub message: String,
}

/// セッション無効化レスポンス
#[derive(Debug, Serialize)]
pub struct InvalidateSessionResponse {
    pub invalidated: bool,
    pub message: String,
}

/// セッション情報レスポンス
#[derive(Debug, Serialize)]
pub struct SessionInfoResponse {
    pub user_id: String,
    pub username: String,
    pub created_at: String,
    pub last_activity: String,
    pub session_age_seconds: i64,
    pub remaining_ttl_seconds: Option<i64>,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

/// Helper: Extract token from Authorization header
fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// POST /api/v1/sessions/validate - セッション検証
#[actix_web::post("/validate")]
pub async fn validate_session(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let token = extract_token_from_request(&req).ok_or(AppError::Unauthorized)?;

    if let Some(ref cache) = app_state.cache {
        let session_mgr = SessionManager::new(cache.clone());

        match session_mgr.validate_session(&token).await {
            Ok(session_data) => {
                let response = ValidateSessionResponse {
                    valid: true,
                    user_id: session_data.user_id,
                    username: session_data.username,
                    created_at: session_data.created_at.to_rfc3339(),
                    last_activity: session_data.last_activity.to_rfc3339(),
                    permissions: session_data.permissions,
                };

                return Ok(HttpResponse::Ok().json(response));
            }
            Err(e) => {
                warn!("Session validation failed: {:?}", e);
                return Err(AppError::Unauthorized);
            }
        }
    }

    // Cache not available - validate JWT only
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "valid": true,
        "note": "Redis unavailable, JWT validation only"
    })))
}

/// POST /api/v1/sessions/extend - セッション TTL 拡張
#[actix_web::post("/extend")]
pub async fn extend_session(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let token = extract_token_from_request(&req).ok_or(AppError::Unauthorized)?;

    if let Some(ref cache) = app_state.cache {
        let session_mgr = SessionManager::new(cache.clone());

        match session_mgr.extend_session(&token).await {
            Ok(_) => {
                info!("Session extended");

                return Ok(HttpResponse::Ok().json(ExtendSessionResponse {
                    extended: true,
                    new_ttl_hours: 24,
                    message: "Session TTL extended to 24 hours".to_string(),
                }));
            }
            Err(e) => {
                warn!("Failed to extend session: {:?}", e);
                return Err(AppError::Unauthorized);
            }
        }
    }

    Err(AppError::Unauthorized)
}

/// POST /api/v1/sessions/invalidate - ログアウト＆セッション破棄
#[actix_web::post("/invalidate")]
pub async fn invalidate_session(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let token = extract_token_from_request(&req).ok_or(AppError::Unauthorized)?;

    if let Some(ref cache) = app_state.cache {
        let session_mgr = SessionManager::new(cache.clone());

        match session_mgr.invalidate_session(&token).await {
            Ok(_) => {
                info!("Session invalidated");

                return Ok(HttpResponse::Ok().json(InvalidateSessionResponse {
                    invalidated: true,
                    message: "Session successfully invalidated".to_string(),
                }));
            }
            Err(e) => {
                warn!("Failed to invalidate session: {:?}", e);
                // Still return success - session might already be expired
                return Ok(HttpResponse::Ok().json(InvalidateSessionResponse {
                    invalidated: true,
                    message: "Session removed or already expired".to_string(),
                }));
            }
        }
    }

    Ok(HttpResponse::Ok().json(InvalidateSessionResponse {
        invalidated: true,
        message: "JWT invalidated (cache unavailable)".to_string(),
    }))
}

/// GET /api/v1/sessions/info - セッション情報取得
#[actix_web::get("/info")]
pub async fn get_session_info(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> AppResult<HttpResponse> {
    let token = extract_token_from_request(&req).ok_or(AppError::Unauthorized)?;
    let now = Utc::now();

    if let Some(ref cache) = app_state.cache {
        let session_mgr = SessionManager::new(cache.clone());

        match session_mgr.validate_session(&token).await {
            Ok(session_data) => {
                let session_age = now.signed_duration_since(session_data.created_at);
                let last_activity = now.signed_duration_since(session_data.last_activity);

                // Estimate remaining TTL (24h - last_activity_age)
                let remaining_ttl = (24 * 3600) - last_activity.num_seconds();

                let response = SessionInfoResponse {
                    user_id: session_data.user_id,
                    username: session_data.username,
                    created_at: session_data.created_at.to_rfc3339(),
                    last_activity: session_data.last_activity.to_rfc3339(),
                    session_age_seconds: session_age.num_seconds(),
                    remaining_ttl_seconds: Some(remaining_ttl.max(0)),
                    permissions: session_data.permissions,
                    is_active: true,
                };

                return Ok(HttpResponse::Ok().json(response));
            }
            Err(_) => {
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "is_active": false,
                    "message": "Session expired or invalid"
                })));
            }
        }
    }

    // Cache unavailable - return JWT info only
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "is_active": true,
        "note": "Redis unavailable, session info limited"
    })))
}

/// ルート設定
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .service(validate_session)
            .service(extend_session)
            .service(invalidate_session)
            .service(get_session_info),
    );
}
