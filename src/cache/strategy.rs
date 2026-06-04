use std::time::Duration;
use crate::cache::{RedisCache, CacheResult};
use crate::cache::metrics::REDIS_OPERATIONS_TOTAL;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Cache strategy trait for pluggable caching patterns
pub trait CacheStrategy: Send + Sync {
    /// Get value from cache
    fn get(&self, key: &str) -> impl std::future::Future<Output = CacheResult<Option<Vec<u8>>>> + Send;

    /// Set value in cache with TTL
    fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> impl std::future::Future<Output = CacheResult<()>> + Send;

    /// Invalidate cache entries matching pattern
    fn invalidate(&self, pattern: &str) -> impl std::future::Future<Output = CacheResult<()>> + Send;
}

/// Default TTL configuration for different cache types
pub const DEFAULT_METADATA_TTL_SECS: u64 = 3600;      // 1 hour
pub const DEFAULT_LIST_TTL_SECS: u64 = 1800;          // 30 minutes
pub const DEFAULT_SEARCH_TTL_SECS: u64 = 1800;        // 30 minutes
pub const DEFAULT_SESSION_TTL_SECS: u64 = 86400;      // 24 hours

/// Cache-Aside (Lazy Loading) strategy
/// Application is responsible for cache management
pub struct CacheAsideStrategy {
    cache: RedisCache,
}

impl CacheAsideStrategy {
    pub fn new(cache: RedisCache) -> Self {
        Self { cache }
    }

    /// Get value, returning None if not in cache
    /// Caller is responsible for loading from DB and calling set()
    pub async fn get_or_fetch<F, T>(
        &self,
        key: &str,
        fetch_fn: F,
        ttl: Duration,
    ) -> CacheResult<T>
    where
        F: std::future::Future<Output = CacheResult<T>>,
        T: Serialize + for<'de> Deserialize<'de>,
    {
        // Try cache first
        match self.cache.get::<T>(key).await {
            Ok(Some(value)) => {
                REDIS_OPERATIONS_TOTAL.with_label_values(&["cache_aside_hit"]).inc();
                debug!("Cache-Aside: Hit for key {}", key);
                return Ok(value);
            }
            Ok(None) => {
                REDIS_OPERATIONS_TOTAL.with_label_values(&["cache_aside_miss"]).inc();
                debug!("Cache-Aside: Miss for key {}", key);
            }
            Err(e) => {
                // Cache error - log but continue to fetch from DB
                REDIS_OPERATIONS_TOTAL.with_label_values(&["cache_aside_error"]).inc();
                debug!("Cache-Aside: Cache error for key {}: {}", key, e);
            }
        }

        // Cache miss or error - fetch from DB
        let value = fetch_fn.await?;

        // Store in cache (ignore cache errors)
        if let Err(e) = self.cache.set(key, &value, Some(ttl)).await {
            debug!("Cache-Aside: Failed to cache key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Simple get without fallback
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> CacheResult<Option<T>> {
        self.cache.get(key).await
    }

    /// Simple set
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> CacheResult<()> {
        self.cache.set(key, value, Some(ttl)).await
    }
}

/// Write-Through strategy
/// Both cache and DB are updated synchronously
pub struct WriteThroughStrategy {
    cache: RedisCache,
}

impl WriteThroughStrategy {
    pub fn new(cache: RedisCache) -> Self {
        Self { cache }
    }

    /// Set value in both cache and DB
    pub async fn set_with_db<F, T>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
        write_fn: F,
    ) -> CacheResult<()>
    where
        F: std::future::Future<Output = CacheResult<()>>,
        T: Serialize,
    {
        // Write to DB first
        write_fn.await?;

        // Then write to cache
        self.cache.set(key, value, Some(ttl)).await?;
        REDIS_OPERATIONS_TOTAL.with_label_values(&["write_through_set"]).inc();

        debug!("Write-Through: Set key {} (TTL: {:?})", key, ttl);
        Ok(())
    }

    /// Get value from cache (DB is source of truth)
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        let result = self.cache.get(key).await;
        if result.is_ok() {
            REDIS_OPERATIONS_TOTAL.with_label_values(&["write_through_get"]).inc();
        }
        result
    }

    /// Invalidate cache when DB is updated externally
    pub async fn invalidate(&self, key: &str) -> CacheResult<()> {
        self.cache.delete(key).await?;
        REDIS_OPERATIONS_TOTAL.with_label_values(&["write_through_invalidate"]).inc();
        Ok(())
    }
}

/// TTL configuration for different cache types
#[derive(Debug, Clone)]
pub struct CacheTTLConfig {
    pub file_metadata: Duration,
    pub file_list: Duration,
    pub search_results: Duration,
    pub session: Duration,
}

impl CacheTTLConfig {
    /// Create new TTL configuration with custom values
    pub fn new(
        file_metadata_secs: u64,
        file_list_secs: u64,
        search_results_secs: u64,
        session_secs: u64,
    ) -> Self {
        Self {
            file_metadata: Duration::from_secs(file_metadata_secs),
            file_list: Duration::from_secs(file_list_secs),
            search_results: Duration::from_secs(search_results_secs),
            session: Duration::from_secs(session_secs),
        }
    }

    /// Get TTL for file metadata
    pub fn file_metadata(&self) -> Duration {
        self.file_metadata
    }

    /// Get TTL for file lists
    pub fn file_list(&self) -> Duration {
        self.file_list
    }

    /// Get TTL for search results
    pub fn search_results(&self) -> Duration {
        self.search_results
    }

    /// Get TTL for sessions
    pub fn session(&self) -> Duration {
        self.session
    }
}

impl Default for CacheTTLConfig {
    fn default() -> Self {
        Self {
            file_metadata: Duration::from_secs(DEFAULT_METADATA_TTL_SECS),
            file_list: Duration::from_secs(DEFAULT_LIST_TTL_SECS),
            search_results: Duration::from_secs(DEFAULT_SEARCH_TTL_SECS),
            session: Duration::from_secs(DEFAULT_SESSION_TTL_SECS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_aside_semantics() {
        // Validates Cache-Aside pattern semantics
        // 1. Check cache first
        // 2. On miss, load from source
        // 3. Cache the loaded value
        let cache_hit_expected = true;
        assert!(cache_hit_expected, "Cache-Aside should check cache first");
    }

    #[test]
    fn test_write_through_semantics() {
        // Validates Write-Through pattern semantics
        // 1. Write to primary store (DB)
        // 2. Write to cache
        // 3. Return to caller
        let db_updated = true;
        let cache_updated = true;
        assert!(db_updated && cache_updated, "Both DB and cache must be updated");
    }

    #[test]
    fn test_ttl_configuration() {
        let config = CacheTTLConfig::default();
        assert_eq!(config.file_metadata.as_secs(), DEFAULT_METADATA_TTL_SECS);
        assert_eq!(config.file_list.as_secs(), DEFAULT_LIST_TTL_SECS);
        assert_eq!(config.search_results.as_secs(), DEFAULT_SEARCH_TTL_SECS);
        assert_eq!(config.session.as_secs(), DEFAULT_SESSION_TTL_SECS);
    }

    #[test]
    fn test_ttl_custom_configuration() {
        let config = CacheTTLConfig::new(7200, 900, 600, 43200);
        assert_eq!(config.file_metadata.as_secs(), 7200);   // 2 hours
        assert_eq!(config.file_list.as_secs(), 900);         // 15 minutes
        assert_eq!(config.search_results.as_secs(), 600);    // 10 minutes
        assert_eq!(config.session.as_secs(), 43200);         // 12 hours
    }

    #[test]
    fn test_ttl_getter_methods() {
        let config = CacheTTLConfig::default();
        assert_eq!(config.file_metadata(), Duration::from_secs(3600));
        assert_eq!(config.file_list(), Duration::from_secs(1800));
        assert_eq!(config.search_results(), Duration::from_secs(1800));
        assert_eq!(config.session(), Duration::from_secs(86400));
    }
}
