pub mod error;
pub mod local_backend;
pub mod s3_backend;
pub mod multipart;
pub mod failover;
pub mod metrics;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_day2;

#[cfg(test)]
mod tests_day3;

#[cfg(test)]
mod tests_day4;

#[cfg(test)]
mod tests_day5;

use async_trait::async_trait;
use bytes::Bytes;
use std::fmt;
use uuid::Uuid;

pub use error::{StorageError, StorageResult};
pub use local_backend::LocalStorageBackend;
pub use s3_backend::S3StorageBackend;
pub use failover::FailoverStorageBackend;
pub use metrics::StorageMetrics;
pub use multipart::{
    MultipartStorageBackend, MultipartUploadInit, MultipartUploadSession, ChunkMetadata,
    UploadProgress,
};

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct StorageUrl {
    pub url: String,
    pub expires_at: Option<i64>,
}

impl fmt::Display for StorageUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store(
        &self,
        data: Bytes,
        metadata: FileMetadata,
    ) -> StorageResult<StorageUrl>;

    async fn retrieve(&self, id: &str) -> StorageResult<Bytes>;

    async fn delete(&self, id: &str) -> StorageResult<()>;

    async fn exists(&self, id: &str) -> StorageResult<bool>;

    async fn health_check(&self) -> StorageResult<()>;
}
