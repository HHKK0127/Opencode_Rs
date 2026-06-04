use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),

    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Invalid cache operation: {0}")]
    InvalidOperation(String),
}

pub type CacheResult<T> = Result<T, CacheError>;
