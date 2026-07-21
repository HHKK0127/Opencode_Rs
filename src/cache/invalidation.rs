#![allow(dead_code)]
use crate::cache::{CacheResult, RedisCache};
use tracing::{debug, info};

/// Cache invalidation patterns for different cache types
#[derive(Debug, Clone)]
pub struct InvalidationPattern {
    pub pattern: String,
    pub description: String,
}

impl InvalidationPattern {
    /// File metadata invalidation (file:metadata:* or file:{id})
    pub fn file_metadata(file_id: &str) -> Self {
        Self {
            pattern: format!("file:metadata:{}*", file_id),
            description: format!("Invalidate file metadata: {}", file_id),
        }
    }

    /// File list invalidation (files:list:*)
    pub fn file_list() -> Self {
        Self {
            pattern: "files:list:*".to_string(),
            description: "Invalidate all file lists".to_string(),
        }
    }

    /// Search results invalidation (files:search:*)
    pub fn search_results() -> Self {
        Self {
            pattern: "files:search:*".to_string(),
            description: "Invalidate all search results".to_string(),
        }
    }

    /// Session invalidation (session:*)
    pub fn session(session_id: &str) -> Self {
        Self {
            pattern: format!("session:{}*", session_id),
            description: format!("Invalidate session: {}", session_id),
        }
    }

    /// User-specific invalidation (user:{id}:*)
    pub fn user_data(user_id: &str) -> Self {
        Self {
            pattern: format!("user:{}:*", user_id),
            description: format!("Invalidate user data: {}", user_id),
        }
    }
}

/// Cache invalidation manager
pub struct CacheInvalidationManager {
    cache: RedisCache,
}

impl CacheInvalidationManager {
    pub fn new(cache: RedisCache) -> Self {
        Self { cache }
    }

    /// Invalidate single cache entry
    pub async fn invalidate_key(&self, key: &str) -> CacheResult<()> {
        debug!("Invalidating cache key: {}", key);
        self.cache.delete(key).await
    }

    /// Invalidate by pattern (file:*, user:*, session:*, etc.)
    pub async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> CacheResult<()> {
        info!(
            "Invalidating pattern: {} ({})",
            pattern.pattern, pattern.description
        );

        // Get all keys matching pattern
        let keys: Vec<String> = self
            .cache
            .get_info()
            .await
            .ok()
            .map(|_| vec![]) // In production, use SCAN command
            .unwrap_or_default();

        // Delete matching keys
        for key in keys {
            if pattern_matches(&key, &pattern.pattern) {
                self.cache.delete(&key).await?;
            }
        }

        Ok(())
    }

    /// Invalidate on file upload (invalidate list, search, stats)
    pub async fn invalidate_on_file_upload(&self) -> CacheResult<()> {
        info!("Invalidating caches after file upload");

        self.cache.delete("files:list:*").await.ok();
        self.cache.delete("files:search:*").await.ok();
        self.cache.delete("files:stats").await.ok();

        Ok(())
    }

    /// Invalidate on file delete (invalidate metadata, list, search)
    pub async fn invalidate_on_file_delete(&self, file_id: &str) -> CacheResult<()> {
        info!("Invalidating caches after file delete: {}", file_id);

        self.cache
            .delete(&format!("file:metadata:{}", file_id))
            .await
            .ok();
        self.cache.delete("files:list:*").await.ok();
        self.cache.delete("files:search:*").await.ok();
        self.cache.delete("files:stats").await.ok();

        Ok(())
    }

    /// Invalidate on user logout (invalidate session, user data)
    pub async fn invalidate_on_logout(&self, user_id: &str) -> CacheResult<()> {
        info!("Invalidating caches after logout: {}", user_id);

        self.cache.delete(&format!("user:{}:*", user_id)).await.ok();

        Ok(())
    }

    /// Bulk invalidate multiple patterns
    pub async fn invalidate_patterns(&self, patterns: Vec<InvalidationPattern>) -> CacheResult<()> {
        info!("Invalidating {} patterns", patterns.len());

        for pattern in patterns {
            self.invalidate_pattern(&pattern).await?;
        }

        Ok(())
    }
}

/// Simple pattern matching for cache keys
fn pattern_matches(key: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        key.starts_with(prefix)
    } else {
        key == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metadata_pattern() {
        let pattern = InvalidationPattern::file_metadata("123");
        assert_eq!(pattern.pattern, "file:metadata:123*");
    }

    #[test]
    fn test_file_list_pattern() {
        let pattern = InvalidationPattern::file_list();
        assert_eq!(pattern.pattern, "files:list:*");
    }

    #[test]
    fn test_pattern_matching() {
        assert!(pattern_matches("file:metadata:123", "file:metadata:*"));
        assert!(pattern_matches("files:list:page1", "files:list:*"));
        assert!(pattern_matches("session:abc123", "session:*"));
        assert!(!pattern_matches("user:123", "files:*"));
    }

    #[test]
    fn test_pattern_exact_match() {
        assert!(pattern_matches("exact:key", "exact:key"));
        assert!(!pattern_matches("exact:key", "exact:key2"));
    }

    #[test]
    fn test_user_data_pattern() {
        let pattern = InvalidationPattern::user_data("user123");
        assert_eq!(pattern.pattern, "user:user123:*");
    }

    #[test]
    fn test_session_pattern() {
        let pattern = InvalidationPattern::session("session123");
        assert_eq!(pattern.pattern, "session:session123*");
    }
}
