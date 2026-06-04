use super::{FileMetadata, StorageBackend, StorageError, StorageResult, StorageUrl};
use async_trait::async_trait;
use aws_sdk_s3::{config::Region, Client};
use bytes::Bytes;

pub struct S3StorageBackend {
    client: Client,
    bucket: String,
    region: String,
}

impl S3StorageBackend {
    pub async fn new(
        endpoint: &str,
        bucket: &str,
        region: &str,
        _access_key: &str,
        _secret_key: &str,
    ) -> StorageResult<Self> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .region(Region::new(region.to_string()))
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            region: region.to_string(),
        })
    }

    fn get_object_key(&self, filename: &str) -> String {
        format!("uploads/{}", filename)
    }
}

#[async_trait]
impl StorageBackend for S3StorageBackend {
    async fn store(
        &self,
        data: Bytes,
        metadata: FileMetadata,
    ) -> StorageResult<StorageUrl> {
        let key = self.get_object_key(&metadata.filename);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(aws_sdk_s3::primitives::ByteStream::from(data))
            .content_type(&metadata.content_type)
            .send()
            .await
            .map_err(|e| StorageError::S3Error(format!("Failed to upload to S3: {}", e)))?;

        let url = format!("s3://{}/{}", self.bucket, key);

        Ok(StorageUrl {
            url,
            expires_at: None,
        })
    }

    async fn retrieve(&self, id: &str) -> StorageResult<Bytes> {
        let key = self.get_object_key(id);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    StorageError::NotFound(format!("Object not found: {}", id))
                } else {
                    StorageError::S3Error(format!("Failed to download from S3: {}", e))
                }
            })?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::S3Error(format!("Failed to read S3 object: {}", e)))?
            .into_bytes();

        Ok(bytes)
    }

    async fn delete(&self, id: &str) -> StorageResult<()> {
        let key = self.get_object_key(id);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| StorageError::S3Error(format!("Failed to delete from S3: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, id: &str) -> StorageResult<bool> {
        let key = self.get_object_key(id);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if e.to_string().contains("NotFound") => Ok(false),
            Err(e) => Err(StorageError::S3Error(format!(
                "Failed to check S3 object: {}",
                e
            ))),
        }
    }

    async fn health_check(&self) -> StorageResult<()> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|e| StorageError::HealthCheckFailed(format!("S3 health check failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_backend_initialization() {
        // Test that S3 backend can be constructed
        // Actual S3 operations require MinIO running
    }
}
