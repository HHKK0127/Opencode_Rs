#![cfg(feature = "postgres")]

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use opencode_poc::app_state::AppState;
use opencode_poc::config::Settings;
use opencode_poc::storage::local_backend::LocalStorageBackend;
use rand_core::OsRng;
use sqlx::postgres::PgPool;
use std::sync::Arc;

/// Connect to PostgreSQL and set up schema for tests
pub async fn setup_test_db() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/opencode_test".to_string()
    });

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL test database. Set DATABASE_URL.");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            original_name TEXT,
            size BIGINT NOT NULL,
            mime_type TEXT,
            checksum TEXT,
            path TEXT,
            user_id TEXT,
            is_public BOOLEAN DEFAULT FALSE,
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create files table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS upload_sessions (
            id TEXT PRIMARY KEY,
            file_id TEXT,
            user_id TEXT,
            total_size BIGINT NOT NULL,
            uploaded_size BIGINT DEFAULT 0,
            chunk_size BIGINT DEFAULT 1048576,
            status TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create upload_sessions table");

    // Create migration history table (best-effort due to concurrent test races)
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            execution_time BIGINT NOT NULL,
            success BOOLEAN NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await;

    pool
}

/// Create test AppState backed by PostgreSQL
pub async fn create_test_app_state() -> AppState {
    let pool = setup_test_db().await;
    let settings = Settings::default();
    let storage: Arc<dyn opencode_poc::storage::StorageBackend> =
        Arc::new(LocalStorageBackend::new("./test_uploads"));
    AppState::new(settings, pool, storage, None)
}

/// Create test user and return (username, password, pool)
pub async fn create_test_user(pool: &PgPool) -> (String, String) {
    let user_id = uuid::Uuid::new_v4().to_string();
    let username = format!("testuser_{}", &user_id[..8]);
    let password = "TestPassword123";

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)")
        .bind(&user_id)
        .bind(&username)
        .bind(&password_hash)
        .execute(pool)
        .await
        .expect("Failed to insert test user");

    (username, password.to_string())
}

/// Generate valid JWT token for testing
pub fn create_test_token(user_id: &str, jwt_secret: &str) -> String {
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = Utc::now();
    let claims = opencode_poc::models::Claims {
        sub: user_id.to_string(),
        username: "test-user".to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(24)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Failed to encode JWT")
}
