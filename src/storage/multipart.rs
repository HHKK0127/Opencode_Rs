use super::StorageResult;
use bytes::Bytes;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MultipartUploadSession {
    pub session_id: String,
    pub file_id: String,
    pub total_size: usize,
    pub chunk_size: usize,
    pub chunks: HashMap<u32, ChunkMetadata>,
}

#[derive(Clone, Debug)]
pub struct ChunkMetadata {
    pub part_number: u32,
    pub etag: String,
    pub size: usize,
    pub offset: usize,
}

pub struct MultipartUploadInit {
    pub file_id: String,
    pub filename: String,
    pub total_size: usize,
    pub content_type: String,
}

#[derive(Clone, Debug)]
pub struct UploadProgress {
    pub session_id: String,
    pub uploaded_bytes: usize,
    pub total_bytes: usize,
    pub completed_parts: u32,
    pub total_parts: u32,
}

impl UploadProgress {
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.uploaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

pub trait MultipartStorageBackend: Send + Sync {
    async fn init_multipart_upload(
        &self,
        init: MultipartUploadInit,
    ) -> StorageResult<String>;

    async fn upload_part(
        &self,
        session_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<String>;

    async fn complete_multipart_upload(
        &self,
        session_id: &str,
        part_etags: Vec<(u32, String)>,
    ) -> StorageResult<String>;

    async fn abort_multipart_upload(&self, session_id: &str) -> StorageResult<()>;

    async fn get_upload_progress(&self, session_id: &str) -> StorageResult<UploadProgress>;

    async fn list_parts(&self, session_id: &str) -> StorageResult<Vec<ChunkMetadata>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_progress_percentage() {
        let progress = UploadProgress {
            session_id: "test".to_string(),
            uploaded_bytes: 500,
            total_bytes: 1000,
            completed_parts: 1,
            total_parts: 2,
        };
        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_upload_progress_zero_total() {
        let progress = UploadProgress {
            session_id: "test".to_string(),
            uploaded_bytes: 0,
            total_bytes: 0,
            completed_parts: 0,
            total_parts: 0,
        };
        assert_eq!(progress.percentage(), 0.0);
    }
}
