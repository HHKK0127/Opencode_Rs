#![allow(dead_code, unused_imports, clippy::all)]

pub mod error;
pub mod invalidation; // Wave 4 Day 12: Cache invalidation
pub mod metrics; // Wave 4 Day 11: Redis metrics
pub mod redis;
pub mod scan;
pub mod session; // Wave 4 Day 14: Session management
pub mod strategy; // Wave 4 Day 12: Cache strategies // Redis SCAN implementation

pub use error::{CacheError, CacheResult};
pub use invalidation::{CacheInvalidationManager, InvalidationPattern};
pub use metrics::{
    register_redis_metrics, REDIS_COMMAND_DURATION_SECONDS, REDIS_CONNECTIONS_ACTIVE,
    REDIS_ERRORS_TOTAL,
};
pub use redis::{RedisCache, RedisCacheConfig};
pub use scan::RedisScanner;
pub use session::{SessionData, SessionManager, UploadSessionData, UploadSessionManager};
pub use strategy::{CacheAsideStrategy, CacheStrategy, CacheTTLConfig, WriteThroughStrategy};
