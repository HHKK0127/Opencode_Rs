use actix_web::{post, web, HttpResponse};
use argon2::PasswordVerifier;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::error::{AppError, AppResult};
use crate::models::{AuthRequest, AuthResponse, Claims, RegisterRequest, RegisterResponse, RefreshTokenRequest, ResetPasswordResponse};
use crate::app_state::AppState;
use uuid::Uuid;

#[post("/auth/login")]
pub async fn login(
    app_state: web::Data<AppState>,
    req: web::Json<AuthRequest>,
) -> AppResult<HttpResponse> {
    let username = &req.username;
    let password = &req.password;

    if username.is_empty() || password.len() < 8 {
        return Err(AppError::BadRequest(
            "Invalid username or password".to_string(),
        ));
    }

    let user = sqlx::query_as::<_, (String, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match user {
        Some((id, password_hash)) => {
            let hash = argon2::PasswordHash::new(&password_hash)
                .map_err(|_| AppError::Internal)?;

            if argon2::Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_err()
            {
                return Err(AppError::Unauthorized);
            }

            // Get JWT settings from cached AppState
            let token = generate_token(
                &id,
                &app_state.settings.auth.jwt_secret,
                app_state.settings.auth.token_expiry_hours,
            )?;
            let expires_in = app_state.settings.auth.token_expiry_hours as i64 * 3600;

            Ok(HttpResponse::Ok().json(AuthResponse {
                token,
                expires_in,
            }))
        }
        None => Err(AppError::Unauthorized),
    }
}

fn generate_token(user_id: &str, jwt_secret: &str, expiry_hours: u32) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(expiry_hours as i64)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
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
    .execute(&app_state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("Username already exists".to_string())
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

#[post("/auth/refresh")]
pub async fn refresh_token(
    app_state: web::Data<AppState>,
    req: web::Json<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    // Get JWT settings from cached AppState (no file I/O)
    let claims = jsonwebtoken::decode::<Claims>(
        &req.token,
        &jsonwebtoken::DecodingKey::from_secret(app_state.settings.auth.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;

    let token = generate_token(
        &claims.sub,
        &app_state.settings.auth.jwt_secret,
        app_state.settings.auth.token_expiry_hours,
    )?;
    let expires_in = app_state.settings.auth.token_expiry_hours as i64 * 3600;

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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(refresh_token)
        .service(reset_password);
}
