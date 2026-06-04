//! StorageBackend trait - 統一されたストレージインターフェース

use async_trait::async_trait;
use bytes::Bytes;
use crate::error::AppError;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub part_number: i32,
    pub etag: String,
}

#[async_trait]
pub trait StorageBackend: Send + Sync + Debug {
    async fn store(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<ObjectMetadata, AppError>;

    async fn retrieve(&self, key: &str) -> Result<(Bytes, ObjectMetadata), AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn exists(&self, key: &str) -> Result<bool, AppError>;
    async fn health_check(&self) -> Result<bool, AppError>;
    async fn init_multipart_upload(&self, key: &str) -> Result<MultipartUpload, AppError>;
    async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: i32,
        data: Bytes,
    ) -> Result<String, AppError>;
    async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> Result<ObjectMetadata, AppError>;
    async fn abort_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
    ) -> Result<(), AppError>;
}

pub type DynStorageBackend = Box<dyn StorageBackend>;
