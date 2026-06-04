use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    config::Credentials,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
    operation::head_object::HeadObjectOutput,
    Client,
};
use std::time::Duration;
use tracing::{error, info};

use crate::config::Settings;
use crate::error::AppError;

type AppResult<T> = Result<T, AppError>;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
    endpoint: String,
}

impl S3Client {
    pub async fn new(settings: &Settings) -> AppResult<Self> {
        let region_provider = RegionProviderChain::default_provider()
            .or_else("us-east-1");

        let region = region_provider.region().await;

        // MinIO用のカスタムエンドポイント設定
        let endpoint_url = settings.storage.s3.endpoint.clone();

        let creds = Credentials::new(
            &settings.storage.s3.access_key,
            &settings.storage.s3.secret_key,
            None,
            None,
            "minio",
        );

        let config = aws_sdk_s3::config::Builder::new()
            .region(region)
            .credentials_provider(creds)
            .endpoint_url(endpoint_url)
            .force_path_style(true)  // MinIO requires path-style
            .build();

        let client = Client::from_conf(config);

        // バケット存在確認・作成
        let bucket = settings.storage.s3.bucket.clone();
        Self::ensure_bucket_exists(&client, &bucket).await?;

        info!("S3Client initialized for bucket: {}", bucket);

        Ok(Self {
            client,
            bucket,
            endpoint: settings.storage.s3.endpoint.clone(),
        })
    }

    async fn ensure_bucket_exists(client: &Client, bucket: &str) -> AppResult<()> {
        match client.head_bucket().bucket(bucket).send().await {
            Ok(_) => {
                info!("Bucket {} exists", bucket);
                Ok(())
            }
            Err(_) => {
                info!("Creating bucket: {}", bucket);
                client
                    .create_bucket()
                    .bucket(bucket)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Failed to create bucket: {}", e);
                        AppError::Internal
                    })?;
                Ok(())
            }
        }
    }

    // 単純アップロード（小ファイル用）
    pub async fn upload_object(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> AppResult<String> {
        let body = ByteStream::from(data);

        let mut builder = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body);

        if let Some(ct) = content_type {
            builder = builder.content_type(ct);
        }

        let result = builder.send().await.map_err(|e| {
            error!("S3 upload failed: {}", e);
            AppError::Internal
        })?;

        let etag = result.e_tag.unwrap_or_default();
        info!("Uploaded object: {} (etag: {})", key, etag);

        Ok(etag)
    }

    // ダウンロード
    pub async fn download_object(&self, key: &str) -> AppResult<Vec<u8>> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("S3 download failed: {}", e);
                AppError::NotFound
            })?;

        let data = result.body.collect().await.map_err(|e| {
            error!("S3 body collection failed: {}", e);
            AppError::Internal
        })?;

        Ok(data.into_bytes().to_vec())
    }

    // 削除
    pub async fn delete_object(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("S3 delete failed: {}", e);
                AppError::Internal
            })?;

        info!("Deleted object: {}", key);
        Ok(())
    }

    // S3オブジェクト存在確認・メタデータ取得
    pub async fn head_object(&self, key: &str) -> AppResult<HeadObjectOutput> {
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("HeadObject failed for {}: {}", key, e);
                AppError::NotFound
            })?;

        info!("HeadObject succeeded for: {}", key);
        Ok(result)
    }

    // Presigned URL生成（アップロード用）
    pub async fn generate_presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: Option<&str>,
    ) -> AppResult<String> {
        let mut builder = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key);

        if let Some(ct) = content_type {
            builder = builder.content_type(ct);
        }

        let presigned = builder
            .presigned(aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(expires_in)
                .build()
                .map_err(|_| AppError::Internal)?)
            .await
            .map_err(|e| {
                error!("Presigned URL generation failed: {}", e);
                AppError::Internal
            })?;

        Ok(presigned.uri().to_string())
    }

    // Presigned URL生成（ダウンロード用）
    pub async fn generate_presigned_get_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> AppResult<String> {
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(expires_in)
                .build()
                .map_err(|_| AppError::Internal)?)
            .await
            .map_err(|e| {
                error!("Presigned URL generation failed: {}", e);
                AppError::Internal
            })?;

        Ok(presigned.uri().to_string())
    }

    // Multipart upload初期化
    pub async fn initiate_multipart_upload(&self, key: &str) -> AppResult<String> {
        let result = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!("Multipart upload initiation failed: {}", e);
                AppError::Internal
            })?;

        let upload_id = result.upload_id.ok_or(AppError::Internal)?;
        info!("Initiated multipart upload: {} (id: {})", key, upload_id);

        Ok(upload_id)
    }

    // パートアップロード
    pub async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: i32,
        data: Vec<u8>,
    ) -> AppResult<CompletedPart> {
        let body = ByteStream::from(data);

        let result = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(body)
            .send()
            .await
            .map_err(|e| {
                error!("Part upload failed: {}", e);
                AppError::Internal
            })?;

        Ok(CompletedPart::builder()
            .e_tag(result.e_tag.unwrap_or_default())
            .part_number(part_number)
            .build())
    }

    // Multipart upload完了
    pub async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> AppResult<String> {
        let completed = CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        let result = self
            .client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(completed)
            .send()
            .await
            .map_err(|e| {
                error!("Multipart completion failed: {}", e);
                AppError::Internal
            })?;

        let etag = result.e_tag.unwrap_or_default();
        info!("Completed multipart upload: {} (etag: {})", key, etag);

        Ok(etag)
    }

    // 公開URL生成（Presigned不要の場合）
    pub fn public_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket, key)
    }

    // Bucketゲッター（移行スクリプト用）
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    // 大容量ファイル最適化アップロード
    pub async fn upload_large_file_optimized(
        &self,
        key: &str,
        file_data: Vec<u8>,
        chunk_size: usize,
    ) -> AppResult<String> {
        let file_size = file_data.len() as u64;

        // 5MB未満は単純アップロード
        if file_size < 5 * 1024 * 1024 {
            return self.upload_object(key, file_data, None).await;
        }

        // 大容量ファイルはマルチパート
        self.multipart_upload_optimized(key, file_data, chunk_size)
            .await
    }

    // マルチパート最適化アップロード
    async fn multipart_upload_optimized(
        &self,
        key: &str,
        file_data: Vec<u8>,
        chunk_size: usize,
    ) -> AppResult<String> {
        let upload_id = self.initiate_multipart_upload(key).await?;

        let mut part_number = 1;
        let mut parts = Vec::new();

        // チャンク単位でアップロード
        for chunk in file_data.chunks(chunk_size) {
            let part = self
                .upload_part(key, &upload_id, part_number, chunk.to_vec())
                .await?;
            parts.push(part);
            part_number += 1;
        }

        self.complete_multipart_upload(key, &upload_id, parts)
            .await
    }

    // キャッシュキー生成（ETag ベース）
    pub fn cache_key_for_etag(key: &str) -> String {
        format!("s3:etag:{}", key)
    }
}

impl Default for S3Client {
    fn default() -> Self {
        // Dummy client for non-S3 mode
        let dummy_config = aws_sdk_s3::config::Builder::new()
            .region(aws_config::Region::new("us-east-1"))
            .endpoint_url("http://localhost:9000")
            .build();

        Self {
            client: Client::from_conf(dummy_config),
            bucket: "default".to_string(),
            endpoint: "http://localhost:9000".to_string(),
        }
    }
}
