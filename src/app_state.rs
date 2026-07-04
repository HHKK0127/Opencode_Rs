use crate::cache::{RedisCache, CacheTTLConfig};
use crate::config::Settings;
use crate::storage::StorageBackend;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state container shared across all requests via web::Data<AppState>
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: PgPool,
    pub storage: Arc<dyn StorageBackend>,
    /// Optional Redis cache (Wave 4)
    pub cache: Option<Arc<RedisCache>>,
    /// Cache TTL configuration
    pub ttl_config: Arc<CacheTTLConfig>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        db: PgPool,
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

    /// Get TTL config
    pub fn get_ttl_config(&self) -> &CacheTTLConfig {
        &self.ttl_config
    }

    /// Get server bind address from cached settings
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.settings.server.host, self.settings.server.port)
    }

    /// Check if cache is available
    #[allow(dead_code)]
    pub fn cache_available(&self) -> bool {
        self.cache.is_some()
    }

    /// Get Redis cache if available
    #[allow(dead_code)]
    pub fn get_cache(&self) -> Option<&Arc<RedisCache>> {
        self.cache.as_ref()
    }

    /// Get database URL from cached settings
    #[allow(dead_code)]
    pub fn db_url(&self) -> &str {
        &self.settings.database.url
    }

    /// Get upload directory from cached settings
    #[allow(dead_code)]
    pub fn upload_dir(&self) -> &str {
        &self.settings.upload.directory
    }
}
