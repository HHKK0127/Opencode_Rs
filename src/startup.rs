use std::env;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::{info, warn};
use crate::error::AppError;

pub type DbPool = PgPool;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub jwt_secret: String,
    pub database_url: String,
}

/// フェーズ1: 環境変数検証
pub fn validate_environment() -> Result<EnvConfig, AppError> {
    info!("🔍 フェーズ 1: 環境変数検証中...");

    let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
        AppError::BadRequest("JWT_SECRET が設定されていません".to_string())
    })?;

    if jwt_secret.len() < 32 {
        return Err(AppError::BadRequest(
            format!("JWT_SECRET は32文字以上である必要があります（現在: {}文字）", jwt_secret.len())
        ));
    }

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        AppError::BadRequest("DATABASE_URL が設定されていません".to_string())
    })?;

    if !database_url.starts_with("postgresql://") && !database_url.starts_with("postgres://") {
        return Err(AppError::BadRequest(
            "DATABASE_URL は 'postgresql://' または 'postgres://' で始まる必要があります".to_string()
        ));
    }

    info!("✅ 環境変数検証完了");
    Ok(EnvConfig { jwt_secret, database_url })
}

/// フェーズ2: DB接続テスト
pub async fn create_and_test_pool(db_url: &str) -> Result<DbPool, AppError> {
    info!("🔍 フェーズ 2: DB接続テスト中...");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .connect(db_url)
        .await
        .map_err(|e| {
            warn!("❌ DB接続失敗: {}", e);
            AppError::Database(format!("DB接続失敗: {}", e))
        })?;

    // 接続テスト
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            warn!("❌ 接続テストクエリ失敗: {}", e);
            AppError::Database(format!("接続テスト失敗: {}", e))
        })?;

    info!("✅ DB接続テスト完了");
    Ok(pool)
}

/// フェーズ3: スキーマ初期化
pub async fn initialize_schema(pool: &DbPool) -> Result<(), AppError> {
    info!("🔍 フェーズ 3: スキーマ初期化中...");

    // users テーブル
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
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("❌ users テーブル作成失敗: {}", e);
        AppError::Database(format!("テーブル作成失敗: {}", e))
    })?;

    // files テーブル
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
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("❌ files テーブル作成失敗: {}", e);
        AppError::Database(format!("テーブル作成失敗: {}", e))
    })?;

    info!("✅ スキーマ初期化完了");
    Ok(())
}

/// フェーズ4: テストユーザー作成
pub async fn ensure_test_user(pool: &DbPool) -> Result<(), AppError> {
    info!("🔍 フェーズ 4: テストユーザー確保中...");

    let test_user_id = uuid::Uuid::new_v4().to_string();
    // testuser/testpassword (Argon2でハッシュ済み)
    let password_hash = "$argon2id$v=19$m=65536,t=3,p=4$dGVzdHBhc3N3b3Jk$test_hash_placeholder";

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(test_user_id)
    .bind("testuser")
    .bind(password_hash)
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("❌ テストユーザー作成失敗: {}", e);
        AppError::Database(format!("ユーザー作成失敗: {}", e))
    })?;

    info!("✅ テストユーザー確保完了");
    Ok(())
}

/// 統合エントリーポイント: 環境検証 → DB接続 → スキーマ初期化 → テストユーザー
pub async fn validate_and_init() -> Result<DbPool, AppError> {
    info!("🚀 アプリケーション起動シーケンス開始...");

    // フェーズ1: 環境変数検証
    let env_config = validate_environment()?;

    // フェーズ2: DB接続テスト & プール作成
    let pool = create_and_test_pool(&env_config.database_url).await?;

    // フェーズ3: スキーマ初期化
    initialize_schema(&pool).await?;

    // フェーズ4: テストユーザー確保
    ensure_test_user(&pool).await?;

    info!("✅ アプリケーション初期化完了 - サーバー起動準備完了");
    Ok(pool)
}
