use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::error::{CacheError, CacheResult};
use super::metrics::{
    REDIS_CACHE_HITS_TOTAL, REDIS_CACHE_MISSES_TOTAL, REDIS_COMMAND_DURATION_SECONDS,
    REDIS_ERRORS_TOTAL, REDIS_OPERATIONS_TOTAL,
};

/// Redis cache using ConnectionManager for automatic reconnection and concurrency
pub struct RedisCache {
    manager: Arc<ConnectionManager>,
    pub config: RedisCacheConfig,
}

#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 10,
            connection_timeout_ms: 5000,
        }
    }
}

impl RedisCache {
    /// Create new Redis cache instance with ConnectionManager (auto-reconnect)
    pub async fn new(config: RedisCacheConfig) -> CacheResult<Self> {
        debug!("Connecting to Redis: {}", config.url);

        let client = redis::Client::open(config.url.as_str())
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        info!(
            url = %config.url,
            max_connections = config.max_connections,
            "Redis ConnectionManager established"
        );

        Ok(Self {
            manager: Arc::new(manager),
            config,
        })
    }

    /// Health check — PING Redis
    pub async fn health_check(&self) -> CacheResult<()> {
        let mut conn = self.manager.as_ref().clone();
        let _: () = redis::cmd("PING")
            .query_async(&mut conn)
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
        let start = Instant::now();
        let mut conn = self.manager.as_ref().clone();

        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                REDIS_ERRORS_TOTAL.with_label_values(&["get"]).inc();
                CacheError::RedisError(e)
            })?;

        REDIS_COMMAND_DURATION_SECONDS
            .with_label_values(&["GET"])
            .observe(start.elapsed().as_secs_f64());
        REDIS_OPERATIONS_TOTAL.with_label_values(&["get"]).inc();

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)?;
                REDIS_CACHE_HITS_TOTAL.inc();
                debug!("Cache hit: {}", key);
                Ok(Some(deserialized))
            }
            None => {
                REDIS_CACHE_MISSES_TOTAL.inc();
                debug!("Cache miss: {}", key);
                Ok(None)
            }
        }
    }

    /// Set value in cache with optional TTL
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let start = Instant::now();
        let serialized = serde_json::to_string(value)?;
        let mut conn = self.manager.as_ref().clone();

        match ttl {
            Some(duration) => {
                let seconds = duration.as_secs() as usize;
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .arg("EX")
                    .arg(seconds)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| {
                        REDIS_ERRORS_TOTAL.with_label_values(&["set"]).inc();
                        CacheError::RedisError(e)
                    })?;
                debug!("Cache set with TTL: {} ({}s)", key, seconds);
            }
            None => {
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| {
                        REDIS_ERRORS_TOTAL.with_label_values(&["set"]).inc();
                        CacheError::RedisError(e)
                    })?;
                debug!("Cache set: {}", key);
            }
        }

        REDIS_COMMAND_DURATION_SECONDS
            .with_label_values(&["SET"])
            .observe(start.elapsed().as_secs_f64());
        REDIS_OPERATIONS_TOTAL.with_label_values(&["set"]).inc();

        Ok(())
    }

    /// Delete key from cache
    pub async fn delete(&self, key: &str) -> CacheResult<()> {
        let start = Instant::now();
        let mut conn = self.manager.as_ref().clone();

        let _: u32 = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                REDIS_ERRORS_TOTAL.with_label_values(&["delete"]).inc();
                CacheError::RedisError(e)
            })?;

        REDIS_COMMAND_DURATION_SECONDS
            .with_label_values(&["DEL"])
            .observe(start.elapsed().as_secs_f64());
        REDIS_OPERATIONS_TOTAL.with_label_values(&["delete"]).inc();

        debug!("Cache deleted: {}", key);
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> CacheResult<bool> {
        let start = Instant::now();
        let mut conn = self.manager.as_ref().clone();

        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                REDIS_ERRORS_TOTAL.with_label_values(&["exists"]).inc();
                CacheError::RedisError(e)
            })?;

        REDIS_COMMAND_DURATION_SECONDS
            .with_label_values(&["EXISTS"])
            .observe(start.elapsed().as_secs_f64());
        REDIS_OPERATIONS_TOTAL.with_label_values(&["exists"]).inc();

        Ok(exists)
    }

    /// Get Redis INFO for monitoring
    pub async fn get_info(&self) -> CacheResult<String> {
        let mut conn = self.manager.as_ref().clone();
        let info: String = redis::cmd("INFO")
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::RedisError(e))?;
        Ok(info)
    }

    #[allow(dead_code)]
    pub async fn flush_all(&self) -> CacheResult<()> {
        let mut conn = self.manager.as_ref().clone();
        let _: () = redis::cmd("FLUSHALL")
            .query_async(&mut conn)
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
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:test_password@127.0.0.1:6379".to_string());
        RedisCache::new(RedisCacheConfig {
            url,
            max_connections: 10,
            connection_timeout_ms: 5000,
        })
        .await
    }

    #[tokio::test]
    async fn test_redis_connection() {
        let redis = create_test_redis().await;
        assert!(redis.is_ok(), "Redis connection should succeed");
    }

    #[tokio::test]
    async fn test_health_check() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        assert!(redis.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_set_get() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:key:wave5";
        let value = serde_json::json!({"message": "hello wave5"});

        assert!(redis.set(key, &value, None).await.is_ok());
        let got: CacheResult<Option<serde_json::Value>> = redis.get(key).await;
        assert_eq!(got.unwrap(), Some(value));
        let _ = redis.delete(key).await;
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:ttl:wave5";
        let value = serde_json::json!({"data": "temporary"});

        let _ = redis.set(key, &value, Some(Duration::from_secs(1))).await;
        assert!(redis.exists(key).await.unwrap());

        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(!redis.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let redis = create_test_redis().await.expect("Redis connection failed");
        let key = "test:delete:wave5";
        let _ = redis.set(key, &serde_json::json!(true), None).await;
        assert!(redis.exists(key).await.unwrap());
        assert!(redis.delete(key).await.is_ok());
        assert!(!redis.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let redis = std::sync::Arc::new(
            create_test_redis().await.expect("Redis connection failed")
        );
        let mut handles = vec![];

        for i in 0..10 {
            let r = redis.clone();
            handles.push(tokio::spawn(async move {
                let key = format!("test:concurrent:{}", i);
                let _ = r.set(&key, &i, Some(Duration::from_secs(5))).await;
                let got: CacheResult<Option<i32>> = r.get(&key).await;
                let _ = r.delete(&key).await;
                got.unwrap() == Some(i)
            }));
        }

        for h in handles {
            assert!(h.await.unwrap(), "Concurrent access should succeed");
        }
    }
}
