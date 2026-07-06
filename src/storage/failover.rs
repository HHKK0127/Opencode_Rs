use super::{FileMetadata, StorageBackend, StorageResult, StorageUrl};
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

pub struct FailoverStorageBackend {
    primary: Arc<dyn StorageBackend>,
    secondary: Arc<dyn StorageBackend>,
}

impl FailoverStorageBackend {
    pub fn new(
        primary: Arc<dyn StorageBackend>,
        secondary: Arc<dyn StorageBackend>,
    ) -> Self {
        Self { primary, secondary }
    }
}

#[async_trait]
impl StorageBackend for FailoverStorageBackend {
    async fn store(
        &self,
        data: Bytes,
        metadata: FileMetadata,
    ) -> StorageResult<StorageUrl> {
        match self.primary.store(data.clone(), metadata.clone()).await {
            Ok(url) => Ok(url),
            Err(_) => {
                eprintln!("Primary storage failed, falling back to secondary");
                self.secondary.store(data, metadata).await
            }
        }
    }

    async fn retrieve(&self, id: &str) -> StorageResult<Bytes> {
        match self.primary.retrieve(id).await {
            Ok(data) => Ok(data),
            Err(_) => {
                eprintln!("Primary retrieval failed, falling back to secondary");
                self.secondary.retrieve(id).await
            }
        }
    }

    async fn delete(&self, id: &str) -> StorageResult<()> {
        match self.primary.delete(id).await {
            Ok(_) => Ok(()),
            Err(_) => {
                eprintln!("Primary delete failed, falling back to secondary");
                self.secondary.delete(id).await
            }
        }
    }

    async fn exists(&self, id: &str) -> StorageResult<bool> {
        match self.primary.exists(id).await {
            Ok(exists) => Ok(exists),
            Err(_) => {
                eprintln!("Primary exists check failed, falling back to secondary");
                self.secondary.exists(id).await
            }
        }
    }

    async fn health_check(&self) -> StorageResult<()> {
        match self.primary.health_check().await {
            Ok(_) => Ok(()),
            Err(_) => {
                eprintln!("Primary health check failed, checking secondary");
                self.secondary.health_check().await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local_backend::LocalStorageBackend;
    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_failover_primary_success() {
        let temp_primary = TempDir::new().unwrap();
        let temp_secondary = TempDir::new().unwrap();

        let primary = Arc::new(LocalStorageBackend::new(temp_primary.path()));
        let secondary = Arc::new(LocalStorageBackend::new(temp_secondary.path()));

        let failover = FailoverStorageBackend::new(primary, secondary);

        let data = Bytes::from("failover test");
        let metadata = FileMetadata {
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: data.len(),
            user_id: Uuid::new_v4(),
        };

        let result = failover.store(data.clone(), metadata).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failover_secondary_on_failure() {
        let temp_secondary = TempDir::new().unwrap();
        let secondary = Arc::new(LocalStorageBackend::new(temp_secondary.path()));

        // Create a mock primary that always fails
        struct FailingBackend;

        #[async_trait]
        impl StorageBackend for FailingBackend {
            async fn store(
                &self,
                _data: Bytes,
                _metadata: FileMetadata,
            ) -> StorageResult<StorageUrl> {
                Err(StorageError::Unknown("Primary failed".to_string()))
            }

            async fn retrieve(&self, _id: &str) -> StorageResult<Bytes> {
                Err(StorageError::NotFound("Not found".to_string()))
            }

            async fn delete(&self, _id: &str) -> StorageResult<()> {
                Err(StorageError::Unknown("Delete failed".to_string()))
            }

            async fn exists(&self, _id: &str) -> StorageResult<bool> {
                Err(StorageError::Unknown("Check failed".to_string()))
            }

            async fn health_check(&self) -> StorageResult<()> {
                Err(StorageError::HealthCheckFailed("Primary down".to_string()))
            }
        }

        let primary = Arc::new(FailingBackend);
        let failover = FailoverStorageBackend::new(primary, secondary);

        let data = Bytes::from("failover test");
        let metadata = FileMetadata {
            filename: "fallback.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: data.len(),
            user_id: Uuid::new_v4(),
        };

        let result = failover.store(data, metadata).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failover_both_failure() {
        struct FailingBackend;

        #[async_trait]
        impl StorageBackend for FailingBackend {
            async fn store(
                &self,
                _data: Bytes,
                _metadata: FileMetadata,
            ) -> StorageResult<StorageUrl> {
                Err(StorageError::Unknown("Failed".to_string()))
            }

            async fn retrieve(&self, _id: &str) -> StorageResult<Bytes> {
                Err(StorageError::Unknown("Failed".to_string()))
            }

            async fn delete(&self, _id: &str) -> StorageResult<()> {
                Err(StorageError::Unknown("Failed".to_string()))
            }

            async fn exists(&self, _id: &str) -> StorageResult<bool> {
                Err(StorageError::Unknown("Failed".to_string()))
            }

            async fn health_check(&self) -> StorageResult<()> {
                Err(StorageError::HealthCheckFailed("Failed".to_string()))
            }
        }

        let primary = Arc::new(FailingBackend);
        let secondary = Arc::new(FailingBackend);

        let failover = FailoverStorageBackend::new(primary, secondary);

        let data = Bytes::from("test");
        let metadata = FileMetadata {
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: data.len(),
            user_id: Uuid::new_v4(),
        };

        let result = failover.store(data, metadata).await;
        assert!(result.is_err());
    }
}
