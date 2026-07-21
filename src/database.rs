#![allow(dead_code)]
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;
use tracing::info;

use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            url: settings.database.url.clone(),
        }
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    info!("Creating database pool: SQLite");

    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(
            &config
                .url
                .replace("sqlite:///", "")
                .replace("sqlite://", ""),
        )
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // 接続テスト
    sqlx::query("SELECT 1").execute(&pool).await?;

    info!("✅ Database pool created successfully");
    Ok(pool)
}

/// ヘルスチェック
pub async fn health_check(pool: &SqlitePool) -> Result<Duration, sqlx::Error> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    let elapsed = start.elapsed();
    info!("Database health check: OK (latency: {:?})", elapsed);
    Ok(elapsed)
}
