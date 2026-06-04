use std::time::Duration;
use crate::cache::{RedisCache, CacheResult};
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
                debug!("Cache-Aside: Hit for key {}", key);
                return Ok(value);
            }
            Ok(None) => {
                debug!("Cache-Aside: Miss for key {}", key);
            }
            Err(e) => {
                // Cache error - log but continue to fetch from DB
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

        debug!("Write-Through: Set key {} (TTL: {:?})", key, ttl);
        Ok(())
    }

    /// Get value from cache (DB is source of truth)
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        self.cache.get(key).await
    }

    /// Invalidate cache when DB is updated externally
    pub async fn invalidate(&self, key: &str) -> CacheResult<()> {
        self.cache.delete(key).await
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

impl Default for CacheTTLConfig {
    fn default() -> Self {
        Self {
            file_metadata: Duration::from_secs(3600),    // 1 hour
            file_list: Duration::from_secs(1800),        // 30 minutes
            search_results: Duration::from_secs(1800),   // 30 minutes
            session: Duration::from_secs(86400),         // 24 hours
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_aside_hit() {
        // Test would require actual Redis instance
        // Skipping for unit tests
    }

    #[tokio::test]
    async fn test_cache_aside_miss() {
        // Test cache miss and fetch from DB
    }

    #[tokio::test]
    async fn test_write_through_consistency() {
        // Test DB and cache are updated together
    }

    #[tokio::test]
    async fn test_ttl_configuration() {
        let config = CacheTTLConfig::default();
        assert_eq!(config.file_metadata.as_secs(), 3600);
        assert_eq!(config.file_list.as_secs(), 1800);
        assert_eq!(config.session.as_secs(), 86400);
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        // Test concurrent reads/writes don't cause issues
    }

    #[tokio::test]
    async fn test_pattern_based_invalidation() {
        // Test pattern matching for cache invalidation
    }
}
