use super::{FileMetadata, StorageBackend, StorageError, StorageResult, StorageUrl};
use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;
use tokio::fs;

pub struct LocalStorageBackend {
    base_path: PathBuf,
}

impl LocalStorageBackend {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn get_file_path(&self, id: &str) -> PathBuf {
        self.base_path.join(id)
    }
}

#[async_trait]
impl StorageBackend for LocalStorageBackend {
    async fn store(&self, data: Bytes, metadata: FileMetadata) -> StorageResult<StorageUrl> {
        let file_path = self.get_file_path(&metadata.filename);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file to disk
        fs::write(&file_path, &data).await?;

        let url = file_path
            .to_str()
            .ok_or_else(|| StorageError::Unknown("Invalid path".to_string()))?
            .to_string();

        Ok(StorageUrl {
            url,
            expires_at: None,
        })
    }

    async fn retrieve(&self, id: &str) -> StorageResult<Bytes> {
        let file_path = self.get_file_path(id);

        if !file_path.exists() {
            return Err(StorageError::NotFound(format!("File not found: {}", id)));
        }

        let data = fs::read(&file_path).await?;
        Ok(Bytes::from(data))
    }

    async fn delete(&self, id: &str) -> StorageResult<()> {
        let file_path = self.get_file_path(id);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        Ok(())
    }

    async fn exists(&self, id: &str) -> StorageResult<bool> {
        let file_path = self.get_file_path(id);
        Ok(file_path.exists())
    }

    async fn health_check(&self) -> StorageResult<()> {
        // Check if base path is accessible
        fs::metadata(&self.base_path).await.map_err(|e| {
            StorageError::HealthCheckFailed(format!("Local storage check failed: {}", e))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_local_backend_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalStorageBackend::new(temp_dir.path());

        let data = Bytes::from("test data");
        let metadata = FileMetadata {
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: data.len(),
            user_id: Uuid::new_v4(),
        };

        let url = backend.store(data.clone(), metadata).await.unwrap();
        assert!(!url.url.is_empty());

        let retrieved = backend.retrieve("test.txt").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_local_backend_delete() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalStorageBackend::new(temp_dir.path());

        let data = Bytes::from("test data");
        let metadata = FileMetadata {
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: data.len(),
            user_id: Uuid::new_v4(),
        };

        backend.store(data, metadata).await.unwrap();
        assert!(backend.exists("test.txt").await.unwrap());

        backend.delete("test.txt").await.unwrap();
        assert!(!backend.exists("test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_local_backend_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalStorageBackend::new(temp_dir.path());

        let result = backend.health_check().await;
        assert!(result.is_ok());
    }
}
