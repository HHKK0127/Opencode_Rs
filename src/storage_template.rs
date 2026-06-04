// Wave 3 Storage Trait Template
// このファイルは src/storage/mod.rs の開発時参考用です

use async_trait::async_trait;
use std::fmt;

/// Storage バックエンド共通インタフェース
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// ファイルを保存
    async fn store(
        &self,
        data: bytes::Bytes,
        metadata: FileMetadata,
    ) -> Result<StorageUrl, StorageError>;

    /// ファイルを取得
    async fn retrieve(&self, id: &str) -> Result<bytes::Bytes, StorageError>;

    /// ファイルを削除
    async fn delete(&self, id: &str) -> Result<(), StorageError>;

    /// ファイル存在確認
    async fn exists(&self, id: &str) -> Result<bool, StorageError>;

    /// バックエンド健全性確認
    async fn health_check(&self) -> Result<(), StorageError>;
}

/// ファイルメタデータ
pub struct FileMetadata {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub user_id: uuid::Uuid,
}

/// ストレージURL（S3キーまたはローカルパス）
pub struct StorageUrl {
    pub url: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// ストレージエラー
#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Unauthorized(String),
    InvalidInput(String),
    InternalError(String),
    NetworkError(String),
    ServiceUnavailable,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "File not found"),
            StorageError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            StorageError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            StorageError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            StorageError::ServiceUnavailable => write!(f, "Service unavailable"),
        }
    }
}

impl std::error::Error for StorageError {}

// ===== LocalStorageBackend (既存) =====

pub struct LocalStorageBackend {
    pub base_path: String,
}

#[async_trait]
impl StorageBackend for LocalStorageBackend {
    async fn store(
        &self,
        data: bytes::Bytes,
        metadata: FileMetadata,
    ) -> Result<StorageUrl, StorageError> {
        // 実装例：ファイルシステムに保存
        // let path = format!("{}/{}/{}", self.base_path, metadata.user_id, metadata.filename);
        // tokio::fs::write(&path, data).await?;
        // Ok(StorageUrl { url: path, expires_at: None })
        todo!("Implement LocalStorageBackend::store")
    }

    async fn retrieve(&self, id: &str) -> Result<bytes::Bytes, StorageError> {
        // 実装例：ファイルシステムから読み込み
        // let data = tokio::fs::read(id).await?;
        // Ok(bytes::Bytes::from(data))
        todo!("Implement LocalStorageBackend::retrieve")
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        // 実装例：ファイル削除
        // tokio::fs::remove_file(id).await?;
        // Ok(())
        todo!("Implement LocalStorageBackend::delete")
    }

    async fn exists(&self, id: &str) -> Result<bool, StorageError> {
        // 実装例：ファイル存在確認
        // Ok(tokio::fs::metadata(id).await.is_ok())
        todo!("Implement LocalStorageBackend::exists")
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        // ローカルストレージは常に健全
        Ok(())
    }
}

// ===== S3StorageBackend (Wave 3) =====

pub struct S3StorageBackend {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
    pub region: String,
}

#[async_trait]
impl StorageBackend for S3StorageBackend {
    async fn store(
        &self,
        data: bytes::Bytes,
        metadata: FileMetadata,
    ) -> Result<StorageUrl, StorageError> {
        // 実装例：S3に put
        // let key = format!("uploads/{}/{}", metadata.user_id, metadata.filename);
        // self.client
        //     .put_object()
        //     .bucket(&self.bucket)
        //     .key(&key)
        //     .body(data.into())
        //     .content_type(metadata.content_type)
        //     .send()
        //     .await?;
        // Ok(StorageUrl { url: key, expires_at: None })
        todo!("Implement S3StorageBackend::store")
    }

    async fn retrieve(&self, id: &str) -> Result<bytes::Bytes, StorageError> {
        // 実装例：S3から get
        // let response = self.client
        //     .get_object()
        //     .bucket(&self.bucket)
        //     .key(id)
        //     .send()
        //     .await?;
        // Ok(response.body.collect().await?.into_bytes())
        todo!("Implement S3StorageBackend::retrieve")
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        // 実装例：S3から delete
        // self.client
        //     .delete_object()
        //     .bucket(&self.bucket)
        //     .key(id)
        //     .send()
        //     .await?;
        // Ok(())
        todo!("Implement S3StorageBackend::delete")
    }

    async fn exists(&self, id: &str) -> Result<bool, StorageError> {
        // 実装例：S3で head
        // match self.client
        //     .head_object()
        //     .bucket(&self.bucket)
        //     .key(id)
        //     .send()
        //     .await
        // {
        //     Ok(_) => Ok(true),
        //     Err(e) if e.to_string().contains("404") => Ok(false),
        //     Err(e) => Err(StorageError::InternalError(e.to_string())),
        // }
        todo!("Implement S3StorageBackend::exists")
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        // 実装例：S3へのヘルスチェック
        // self.client
        //     .head_bucket()
        //     .bucket(&self.bucket)
        //     .send()
        //     .await
        //     .map_err(|e| StorageError::ServiceUnavailable)?;
        // Ok(())
        todo!("Implement S3StorageBackend::health_check")
    }
}

// ===== FailoverStorageBackend (Wave 3) =====

pub struct FailoverStorageBackend {
    pub primary: Box<dyn StorageBackend>,
    pub secondary: Box<dyn StorageBackend>,
}

#[async_trait]
impl StorageBackend for FailoverStorageBackend {
    async fn store(
        &self,
        data: bytes::Bytes,
        metadata: FileMetadata,
    ) -> Result<StorageUrl, StorageError> {
        // Primary に try、失敗したら Secondary に fallback
        match self.primary.store(data.clone(), metadata.clone()).await {
            Ok(url) => Ok(url),
            Err(_) => {
                // Primary失敗 → Secondary に フェイルオーバー
                self.secondary.store(data, metadata).await
            }
        }
    }

    async fn retrieve(&self, id: &str) -> Result<bytes::Bytes, StorageError> {
        match self.primary.retrieve(id).await {
            Ok(data) => Ok(data),
            Err(_) => self.secondary.retrieve(id).await,
        }
    }

    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        // 両方から削除（Primary失敗しても Secondary は実行）
        let primary_result = self.primary.delete(id).await;
        let secondary_result = self.secondary.delete(id).await;

        primary_result.or(secondary_result)
    }

    async fn exists(&self, id: &str) -> Result<bool, StorageError> {
        match self.primary.exists(id).await {
            Ok(exists) => Ok(exists),
            Err(_) => self.secondary.exists(id).await,
        }
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        // Primary がOKなら成功（Secondary は backup用）
        self.primary.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_trait_compilation() {
        // コンパイル確認テスト
        // 実装時に具体的なテストに置き換え
    }
}
