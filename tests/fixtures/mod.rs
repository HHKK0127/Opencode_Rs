use opencode_poc::app_state::AppState;
use opencode_poc::config::Settings;
use opencode_poc::storage::s3_client::S3Client;
use sqlx::sqlite::SqlitePool;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand_core::OsRng;

/// Setup in-memory test database with schema
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory test database");

    // Create tables
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
            size INTEGER NOT NULL,
            path TEXT NOT NULL,
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create files table");

    pool
}

/// Create test AppState with in-memory database
pub async fn create_test_app_state() -> AppState {
    let pool = setup_test_db().await;
    let settings = Settings::default();
    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client for tests");
    AppState::new(settings, pool, s3_client)
}

/// Create test user and return (username, password)
pub async fn create_test_user(pool: &SqlitePool) -> (String, String) {
    let user_id = uuid::Uuid::new_v4().to_string();
    let username = format!("testuser_{}", &user_id[..8]);
    let password = "TestPassword123";

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)"
    )
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
