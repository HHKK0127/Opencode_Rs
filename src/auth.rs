use actix_web::{post, web, HttpResponse};
use argon2::PasswordVerifier;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::{AuthRequest, AuthResponse, Claims, RegisterRequest, RegisterResponse, RefreshTokenRequest, ResetPasswordResponse};
use uuid::Uuid;

const JWT_SECRET: &str = "test_secret_key_for_poc_verification";

#[post("/api/v1/auth/login")]
pub async fn login(
    pool: web::Data<SqlitePool>,
    req: web::Json<AuthRequest>,
) -> AppResult<HttpResponse> {
    let username = &req.username;
    let password = &req.password;

    // Validate input
    if username.is_empty() || password.len() < 8 {
        return Err(AppError::BadRequest(
            "Invalid username or password".to_string(),
        ));
    }

    // Query user
    let user = sqlx::query_as::<_, (String, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match user {
        Some((id, password_hash)) => {
            // Verify password
            let hash = argon2::PasswordHash::new(&password_hash)
                .map_err(|_| AppError::Internal)?;

            if argon2::Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_err()
            {
                return Err(AppError::Unauthorized);
            }

            // Generate JWT
            let token = generate_token(&id)?;

            Ok(HttpResponse::Ok().json(AuthResponse {
                token,
                expires_in: 86400,
            }))
        }
        None => Err(AppError::Unauthorized),
    }
}

fn generate_token(user_id: &str) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(24)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|_| AppError::Internal)
}

#[post("/api/v1/auth/register")]
pub async fn register(
    pool: web::Data<SqlitePool>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    let username = &req.username;
    let password = &req.password;

    // Validate input
    if username.is_empty() || password.len() < 8 {
        return Err(AppError::BadRequest(
            "Username and password (min 8 chars) required".to_string(),
        ));
    }

    // Hash password
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand_core::OsRng;

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(username)
    .bind(&password_hash)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::BadRequest("Username already exists".to_string())
        } else {
            AppError::Database(e.to_string())
        }
    })?;

    Ok(HttpResponse::Created().json(RegisterResponse {
        id: user_id,
        username: username.clone(),
        created_at: now,
    }))
}

#[post("/api/v1/auth/refresh")]
pub async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    // Verify the token
    let claims = jsonwebtoken::decode::<Claims>(
        &req.token,
        &jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;

    // Generate new token
    let token = generate_token(&claims.sub)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        expires_in: 86400,
    }))
}

#[post("/api/v1/auth/reset-password")]
pub async fn reset_password(
    _pool: web::Data<SqlitePool>,
    _req: web::Json<crate::models::ResetPasswordRequest>,
) -> AppResult<HttpResponse> {
    // This is a stub implementation
    // In production, this would send an email with reset link
    Ok(HttpResponse::Ok().json(ResetPasswordResponse {
        message: "Password reset email sent (placeholder)".to_string(),
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(refresh_token)
        .service(reset_password);
}
