#![allow(dead_code)]
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct S3CacheEntry {
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub expires: DateTime<Utc>,
}

#[derive(Clone)]
pub struct S3Cache {
    entries: Arc<RwLock<HashMap<String, S3CacheEntry>>>,
    ttl_seconds: i64,
}

impl S3Cache {
    pub fn new(ttl_seconds: i64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl_seconds,
        }
    }

    pub async fn get(&self, key: &str) -> Option<S3CacheEntry> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            if entry.expires > Utc::now() {
                return Some(entry.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, etag: String) {
        let now = Utc::now();
        let expires = now + Duration::seconds(self.ttl_seconds);

        let mut entries = self.entries.write().await;
        entries.insert(
            key,
            S3CacheEntry {
                etag,
                last_modified: now,
                expires,
            },
        );
    }

    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = S3Cache::new(3600);
        cache.set("key1".to_string(), "etag1".to_string()).await;

        let entry = cache.get("key1").await;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().etag, "etag1");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = S3Cache::new(-1); // 既に期限切れ
        cache.set("key1".to_string(), "etag1".to_string()).await;

        let entry = cache.get("key1").await;
        assert!(entry.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = S3Cache::new(3600);
        cache.set("key1".to_string(), "etag1".to_string()).await;
        cache.invalidate("key1").await;

        let entry = cache.get("key1").await;
        assert!(entry.is_none());
    }
}
