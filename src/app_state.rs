use crate::cache::{RedisCache, CacheAsideStrategy, WriteThroughStrategy, CacheTTLConfig, CacheInvalidationManager};
use crate::config::Settings;
use crate::storage::StorageBackend;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Application state container
/// Caches configuration, database connection pool, storage backend, and Redis cache
/// Shared across all requests via web::Data<AppState>
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: SqlitePool,
    /// Unified storage backend (Local for dev, S3/MinIO for prod)
    pub storage: Arc<dyn StorageBackend>,
    /// Optional Redis cache (Wave 4)
    pub cache: Option<Arc<RedisCache>>,
    /// Cache TTL configuration
    pub ttl_config: Arc<CacheTTLConfig>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        db: SqlitePool,
        storage: Arc<dyn StorageBackend>,
        cache: Option<Arc<RedisCache>>,
    ) -> Self {
        Self {
            settings: Arc::new(settings),
            db,
            storage,
            cache,
            ttl_config: Arc::new(CacheTTLConfig::default()),
        }
    }

    /// Check if cache is available
    pub fn cache_available(&self) -> bool {
        self.cache.is_some()
    }

    /// Get Redis cache if available
    pub fn get_cache(&self) -> Option<&Arc<RedisCache>> {
        self.cache.as_ref()
    }

    /// Get TTL config
    pub fn get_ttl_config(&self) -> &CacheTTLConfig {
        &self.ttl_config
    }

    /// Get server bind address from cached settings
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.settings.server.host, self.settings.server.port)
    }

    /// Get database path from cached settings
    pub fn db_path(&self) -> &str {
        &self.settings.database.path
    }

    /// Get upload directory from cached settings
    pub fn upload_dir(&self) -> &str {
        &self.settings.upload.directory
    }
}
