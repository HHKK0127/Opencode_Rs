use crate::cache::{RedisCache, CacheResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

/// Cache integration helper for API endpoints
pub struct CacheIntegration {
    cache: Option<RedisCache>,
}

impl CacheIntegration {
    pub fn new(cache: Option<RedisCache>) -> Self {
        Self { cache }
    }

    /// Get from cache or fetch from DB
    pub async fn get_or_fetch<T, F>(
        &self,
        cache_key: &str,
        fetch_fn: F,
        ttl: Duration,
    ) -> CacheResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: std::future::Future<Output = CacheResult<T>>,
    {
        // Try cache first
        if let Some(cache) = &self.cache {
            match cache.get::<T>(cache_key).await {
                Ok(Some(value)) => {
                    debug!("Cache hit: {}", cache_key);
                    return Ok(value);
                }
                Ok(None) => {
                    debug!("Cache miss: {}", cache_key);
                }
                Err(e) => {
                    warn!("Cache error on get ({}): {}", cache_key, e);
                }
            }
        }

        // Cache miss or no cache - fetch from DB
        let value = fetch_fn.await?;

        // Try to cache the result
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.set(cache_key, &value, Some(ttl)).await {
                warn!("Failed to cache {}: {}", cache_key, e);
            }
        }

        Ok(value)
    }

    /// Invalidate cache patterns
    pub async fn invalidate_patterns(&self, patterns: Vec<&str>) -> CacheResult<()> {
        if let Some(cache) = &self.cache {
            for pattern in patterns {
                if let Err(e) = cache.delete(pattern).await {
                    warn!("Failed to invalidate pattern {}: {}", pattern, e);
                }
            }
        }
        Ok(())
    }

    /// Invalidate single cache key
    pub async fn invalidate_key(&self, key: &str) -> CacheResult<()> {
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.delete(key).await {
                warn!("Failed to invalidate key {}: {}", key, e);
            }
        }
        Ok(())
    }

    /// Check cache statistics
    pub async fn get_cache_info(&self) -> CacheResult<Option<String>> {
        if let Some(cache) = &self.cache {
            match cache.get_info().await {
                Ok(info) => return Ok(Some(info)),
                Err(e) => return Err(e),
            }
        }
        Ok(None)
    }
}

/// Cache key generation utilities
pub mod cache_keys {
    /// File metadata cache key
    pub fn file_metadata(file_id: &str) -> String {
        format!("file:metadata:{}", file_id)
    }

    /// File list cache key
    pub fn file_list(page: u32, per_page: u32) -> String {
        format!("files:list:{}:{}", page, per_page)
    }

    /// Search results cache key
    pub fn search_results(query_hash: &str) -> String {
        format!("files:search:{}", query_hash)
    }

    /// File statistics cache key
    pub fn file_stats() -> String {
        "files:stats".to_string()
    }

    /// Generate query hash for caching
    pub fn query_hash(query: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Cache invalidation patterns
pub mod invalidation_patterns {
    /// Patterns to invalidate on file upload
    pub fn on_file_upload() -> Vec<&'static str> {
        vec!["files:list:*", "files:search:*", "files:stats"]
    }

    /// Patterns to invalidate on file delete
    pub fn on_file_delete(file_id: &str) -> Vec<String> {
        vec![
            format!("file:metadata:{}", file_id),
            "files:list:*".to_string(),
            "files:search:*".to_string(),
            "files:stats".to_string(),
        ]
    }

    /// Patterns to invalidate on search filter change
    pub fn on_search_filter_change() -> Vec<&'static str> {
        vec!["files:search:*"]
    }
}

/// TTL constants for different cache types
pub mod ttls {
    use std::time::Duration;

    pub fn file_metadata() -> Duration {
        Duration::from_secs(3600) // 1 hour
    }

    pub fn file_list() -> Duration {
        Duration::from_secs(1800) // 30 minutes
    }

    pub fn search_results() -> Duration {
        Duration::from_secs(1800) // 30 minutes
    }

    pub fn file_stats() -> Duration {
        Duration::from_secs(1800) // 30 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        assert_eq!(cache_keys::file_metadata("123"), "file:metadata:123");
        assert_eq!(cache_keys::file_list(1, 20), "files:list:1:20");
        assert_eq!(cache_keys::file_stats(), "files:stats");
    }

    #[test]
    fn test_query_hash() {
        let hash1 = cache_keys::query_hash("test query");
        let hash2 = cache_keys::query_hash("test query");
        assert_eq!(hash1, hash2, "Same query should produce same hash");

        let hash3 = cache_keys::query_hash("different query");
        assert_ne!(hash1, hash3, "Different query should produce different hash");
    }

    #[test]
    fn test_invalidation_patterns() {
        let upload_patterns = invalidation_patterns::on_file_upload();
        assert_eq!(upload_patterns.len(), 3);
        assert!(upload_patterns.contains(&"files:list:*"));

        let delete_patterns = invalidation_patterns::on_file_delete("123");
        assert_eq!(delete_patterns.len(), 4);
        assert!(delete_patterns.contains(&"file:metadata:123".to_string()));
    }

    #[test]
    fn test_ttl_configurations() {
        assert_eq!(ttls::file_metadata().as_secs(), 3600);
        assert_eq!(ttls::file_list().as_secs(), 1800);
        assert_eq!(ttls::search_results().as_secs(), 1800);
        assert_eq!(ttls::file_stats().as_secs(), 1800);
        assert!(ttls::file_metadata() > ttls::file_list());
    }
}
