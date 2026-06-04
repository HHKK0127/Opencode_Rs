use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("S3 error: {0}")]
    S3Error(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown storage error: {0}")]
    Unknown(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
