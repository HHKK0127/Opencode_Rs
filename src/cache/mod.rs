pub mod error;
pub mod redis;
pub mod strategy;      // Wave 4 Day 12: Cache strategies
pub mod invalidation;  // Wave 4 Day 12: Cache invalidation
pub mod session;       // Wave 4 Day 14: Session management

pub use error::{CacheError, CacheResult};
pub use redis::{RedisCache, RedisCacheConfig};
pub use strategy::{CacheStrategy, CacheAsideStrategy, WriteThroughStrategy, CacheTTLConfig};
pub use invalidation::{InvalidationPattern, CacheInvalidationManager};
pub use session::{SessionData, SessionManager};
