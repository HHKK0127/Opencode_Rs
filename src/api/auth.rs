use actix_web::{post, web, HttpResponse};
use argon2::{Argon2, PasswordVerifier, PasswordHasher, Algorithm, Version, Params};
use argon2::password_hash::SaltString;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};
use crate::models::{AuthRequest, AuthResponse, Claims, RegisterRequest, RegisterResponse, RefreshTokenRequest, ResetPasswordResponse};
use crate::app_state::AppState;
use crate::cache::session::SessionManager;
use crate::config::{JWT_SECRET, JWT_EXPIRATION_HOURS, ARGON2_MEMORY_COST, ARGON2_TIME_COST, ARGON2_PARALLELISM};
use uuid::Uuid;

/// Argon2id インスタンス生成（強化パラメータ）
fn create_argon2() -> AppResult<Argon2<'static>> {
    let params = Params::new(
        *ARGON2_MEMORY_COST,
        *ARGON2_TIME_COST,
        *ARGON2_PARALLELISM,
        Some(32),
    ).map_err(|_| AppError::Internal)?;

    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

#[post("/auth/login")]
pub async fn login(
    app_state: web::Data<AppState>,
    req: web::Json<AuthRequest>,
) -> AppResult<HttpResponse> {
    let username = &req.username;
    let password = &req.password;

    if username.is_empty() || password.len() < 8 {
        return Err(AppError::BadRequest(
            "Invalid username or password (min 8 chars)".to_string(),
        ));
    }

    let user = sqlx::query_as::<_, (String, String)>(
        "SELECT id, password_hash FROM users WHERE username = $1"
    )
    .bind(username)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match user {
        Some((id, password_hash)) => {
            let hash = argon2::PasswordHash::new(&password_hash)
                .map_err(|_| AppError::Internal)?;

            if create_argon2()?
                .verify_password(password.as_bytes(), &hash)
                .is_err()
            {
                warn!("Failed login attempt for user: {}", username);
                return Err(AppError::Unauthorized);
            }

            let token = generate_token(&id, username)?;
            let expires_in = *JWT_EXPIRATION_HOURS as i64 * 3600;

            // Redisセッション作成
            if let Some(cache) = &app_state.cache {
                let session_mgr = SessionManager::new(cache.clone());
                let permissions = vec!["read".to_string(), "write".to_string()];

                if let Err(e) = session_mgr.create_session(&token, &id, username, permissions).await {
                    warn!("Failed to create session: {:?}", e);
                }
            }

            info!("User logged in: {}", username);

            Ok(HttpResponse::Ok().json(AuthResponse {
                token,
                expires_in,
            }))
        }
        None => {
            warn!("Login attempt for non-existent user: {}", username);
            Err(AppError::Unauthorized)
        }
    }
}

fn generate_token(user_id: &str, username: &str) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(*JWT_EXPIRATION_HOURS as i64)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|_| AppError::Internal)
}

#[post("/auth/register")]
pub async fn register(
    app_state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    let username = &req.username;
    let password = &req.password;

    if username.is_empty() || password.len() < 8 {
        return Err(AppError::BadRequest(
            "Username and password (min 8 chars) required".to_string(),
        ));
    }

    // パスワード強度検証
    validate_password_strength(password)?;

    let salt = SaltString::generate(OsRng);
    let argon2 = create_argon2()?;
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(&user_id)
    .bind(username)
    .bind(&password_hash)
    .bind(&now)
    .execute(&app_state.db)
    .await
    .map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("unique") || msg.contains("duplicate") {
            AppError::Conflict("Username already exists".to_string())
        } else {
            AppError::Database(e.to_string())
        }
    })?;

    info!("User registered: {}", username);

    Ok(HttpResponse::Created().json(RegisterResponse {
        id: user_id,
        username: username.clone(),
        created_at: now,
    }))
}

/// パスワード強度検証
fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }
    
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if !has_upper || !has_lower || !has_digit || !has_special {
        return Err(AppError::BadRequest(
            "Password must contain uppercase, lowercase, digit, and special character".to_string()
        ));
    }
    
    Ok(())
}

#[post("/auth/refresh")]
pub async fn refresh_token(
    _app_state: web::Data<AppState>,
    req: web::Json<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let claims = jsonwebtoken::decode::<Claims>(
        &req.token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;

    let token = generate_token(&claims.sub, &claims.username)?;
    let expires_in = *JWT_EXPIRATION_HOURS as i64 * 3600;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        expires_in,
    }))
}

#[post("/auth/reset-password")]
pub async fn reset_password(
    _app_state: web::Data<AppState>,
    _req: web::Json<crate::models::ResetPasswordRequest>,
) -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(ResetPasswordResponse {
        message: "Password reset email sent (placeholder)".to_string(),
    }))
}

#[post("/auth/logout")]
pub async fn logout(
    app_state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string();

    if let Some(cache) = &app_state.cache {
        let session_mgr = SessionManager::new(cache.clone());
        if !token.is_empty() {
            let _ = session_mgr.invalidate_session(&token).await;
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "logged_out",
        "message": "Successfully logged out"
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(refresh_token)
        .service(reset_password)
        .service(logout);
}
