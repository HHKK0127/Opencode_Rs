use sqlx::AnyPool;
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

pub async fn create_pool(config: &DatabaseConfig) -> Result<AnyPool, sqlx::Error> {
    info!("Creating database pool: {}", 
        if config.url.starts_with("sqlite://") { "SQLite" } else { "PostgreSQL" }
    );

    // シンプルに接続
    let pool = sqlx::AnyPool::connect(&config.url).await?;

    // 接続テスト
    sqlx::query("SELECT 1").execute(&pool).await?;

    info!("✅ Database pool created successfully");
    Ok(pool)
}

/// ヘルスチェック
pub async fn health_check(pool: &AnyPool) -> Result<Duration, sqlx::Error> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    let elapsed = start.elapsed();
    info!("Database health check: OK (latency: {:?})", elapsed);
    Ok(elapsed)
}
