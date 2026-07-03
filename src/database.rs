use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
}

impl DatabaseConfig {
    /// Settingsから検証済みDatabaseConfigを構築
    pub fn from_settings(settings: &Settings) -> Self {
        // 設定値の検証と正規化
        let max = settings.database.max_connections.max(1);
        let min = settings.database.min_connections.min(max);

        if settings.database.min_connections > settings.database.max_connections {
            warn!(
                "min_connections({}) > max_connections({}), adjusting min to {}",
                settings.database.min_connections,
                settings.database.max_connections,
                min
            );
        }

        let acquire_timeout = Duration::from_secs(
            settings.database.acquire_timeout_secs.max(1)
        );

        Self {
            url: settings.database.url.clone(),
            max_connections: max,
            min_connections: min,
            acquire_timeout,
            max_lifetime: Some(Duration::from_secs(3600)), // 1時間
            idle_timeout: Some(Duration::from_secs(600)),   // 10分
        }
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    info!(
        "Creating database pool: max={}, min={}, timeout={:?}",
        config.max_connections, config.min_connections, config.acquire_timeout
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .max_lifetime(config.max_lifetime)
        .idle_timeout(config.idle_timeout)
        .test_before_acquire(true)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // PostgreSQL接続後の初期化
                sqlx::query("SET application_name = 'opencode_rs'")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&config.url)
        .await?;

    // 接続テスト
    let row: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, 1, "Connection test failed");

    info!("✅ Database pool created successfully");
    Ok(pool)
}

/// ヘルスチェック（レイテンシ付き）
pub async fn health_check(pool: &PgPool) -> Result<Duration, sqlx::Error> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    let elapsed = start.elapsed();
    info!("Database health check: OK (latency: {:?})", elapsed);
    Ok(elapsed)
}
