use redis::aio::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use super::error::{CacheError, CacheResult};

/// Redis connection pool wrapper
pub struct RedisCache {
    conn: Arc<Mutex<Connection>>,
    config: RedisCacheConfig,
}

#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
}

impl RedisCache {
    /// Create new Redis cache instance
    pub async fn new(config: RedisCacheConfig) -> CacheResult<Self> {
        debug!("Connecting to Redis: {}", config.url);

        let redis_client = redis::Client::open(config.url.as_str())
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let conn = redis_client
            .get_async_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        info!("Redis connection established");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        })
    }

    /// Health check - ping Redis server
    pub async fn health_check(&self) -> CacheResult<()> {
        let mut conn = self.conn.lock().await;
        let _: () = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        debug!("Redis health check passed");
        Ok(())
    }

    /// Get value from cache
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> CacheResult<Option<T>> {
        let mut conn = self.conn.lock().await;
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)?;
                debug!("Cache hit: {}", key);
                Ok(Some(deserialized))
            }
            None => {
                debug!("Cache miss: {}", key);
                Ok(None)
            }
        }
    }

    /// Set value in cache with TTL
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_string(value)?;
        let mut conn = self.conn.lock().await;

        match ttl {
            Some(duration) => {
                let seconds = duration.as_secs() as usize;
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .arg("EX")
                    .arg(seconds)
                    .query_async(&mut *conn)
                    .await
                    .map_err(|e| CacheError::RedisError(e))?;
                debug!("Cache set with TTL: {} ({}s)", key, seconds);
            }
            None => {
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .query_async(&mut *conn)
                    .await
                    .map_err(|e| CacheError::RedisError(e))?;
                debug!("Cache set: {}", key);
            }
        }

        Ok(())
    }

    /// Delete key from cache
    pub async fn delete(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.conn.lock().await;
        let _: u32 = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;

        debug!("Cache deleted: {}", key);
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> CacheResult<bool> {
        let mut conn = self.conn.lock().await;
        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;

        Ok(exists)
    }

    /// Get Redis info for monitoring
    pub async fn get_info(&self) -> CacheResult<String> {
        let mut conn = self.conn.lock().await;
        let info: String = redis::cmd("INFO")
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;

        Ok(info)
    }

    /// Flush all keys (dangerous - use with caution)
    #[allow(dead_code)]
    pub async fn flush_all(&self) -> CacheResult<()> {
        let mut conn = self.conn.lock().await;
        let _: () = redis::cmd("FLUSHALL")
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;

        warn!("Redis cache flushed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_redis() -> CacheResult<RedisCache> {
        let config = RedisCacheConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 10,
            connection_timeout_ms: 5000,
        };
        RedisCache::new(config).await
    }

    #[tokio::test]
    async fn test_redis_connection() {
        let redis = create_test_redis().await;
        assert!(redis.is_ok(), "Redis connection should succeed");
    }

    #[tokio::test]
    async fn test_health_check() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let result = redis.health_check().await;
        assert!(result.is_ok(), "Health check should pass");
    }

    #[tokio::test]
    async fn test_set_get() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:key";
        let value = serde_json::json!({"message": "hello"});

        let set_result = redis.set(key, &value, None).await;
        assert!(set_result.is_ok(), "Set should succeed");

        let get_result: CacheResult<Option<serde_json::Value>> = redis.get(key).await;
        assert!(get_result.is_ok(), "Get should succeed");
        assert_eq!(get_result.unwrap(), Some(value));

        let _ = redis.delete(key).await;
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:ttl";
        let value = serde_json::json!({"data": "temporary"});

        let _ = redis.set(key, &value, Some(Duration::from_secs(1))).await;

        let exists = redis.exists(key).await.unwrap();
        assert!(exists, "Key should exist immediately after set");

        tokio::time::sleep(Duration::from_millis(1100)).await;

        let exists = redis.exists(key).await.unwrap();
        assert!(!exists, "Key should be expired after TTL");
    }

    #[tokio::test]
    async fn test_delete() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:delete";
        let value = serde_json::json!({"delete": true});

        let _ = redis.set(key, &value, None).await;
        assert!(redis.exists(key).await.unwrap(), "Key should exist");

        let delete_result = redis.delete(key).await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        assert!(
            !redis.exists(key).await.unwrap(),
            "Key should not exist after delete"
        );
    }
}
