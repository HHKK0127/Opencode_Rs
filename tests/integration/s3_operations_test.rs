//! Integration tests for S3 storage operations
//! Tests StorageBackend trait implementations (upload, download, delete, exists)

use opencode_poc::storage::{S3Client, S3StorageBackend, StorageBackend};
use std::sync::Arc;

/// Helper function to create test S3StorageBackend
/// Uses environment variables: S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY, S3_BUCKET
async fn create_test_backend() -> Arc<dyn StorageBackend> {
    // Note: Tests require running MinIO or AWS S3 environment
    // Set env vars before running:
    //   export S3_ENDPOINT=http://localhost:9000
    //   export S3_ACCESS_KEY=minioadmin
    //   export S3_SECRET_KEY=minioadmin123

    let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let access_key = std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin123".to_string());
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "test-uploads".to_string());

    // Create settings dynamically for testing
    let settings = opencode_poc::config::Settings {
        storage: opencode_poc::config::StorageConfig {
            s3: opencode_poc::config::S3Config {
                endpoint,
                access_key,
                secret_key,
                bucket,
                region: "us-east-1".to_string(),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to create S3Client for testing");

    Arc::new(S3StorageBackend::new(s3_client))
}

#[actix_web::test]
async fn test_store_and_retrieve_file() {
    let backend = create_test_backend().await;
    let test_key = "test/file-upload.txt";
    let test_data = b"Hello, S3!".to_vec();

    // Store
    let etag = backend
        .store(test_key, test_data.clone(), Some("text/plain"))
        .await
        .expect("Failed to store file");

    assert!(!etag.is_empty(), "ETag should not be empty");

    // Retrieve
    let retrieved = backend
        .retrieve(test_key)
        .await
        .expect("Failed to retrieve file");

    assert_eq!(retrieved, test_data, "Retrieved data should match uploaded data");

    // Cleanup
    backend.delete(test_key).await.ok();
}

#[actix_web::test]
async fn test_delete_file() {
    let backend = create_test_backend().await;
    let test_key = "test/file-delete.txt";
    let test_data = b"File to delete".to_vec();

    // Store
    backend
        .store(test_key, test_data, Some("text/plain"))
        .await
        .expect("Failed to store file");

    // Verify exists
    let exists = backend
        .exists(test_key)
        .await
        .expect("Failed to check existence");
    assert!(exists, "File should exist after upload");

    // Delete
    backend
        .delete(test_key)
        .await
        .expect("Failed to delete file");

    // Verify deleted
    let exists = backend
        .exists(test_key)
        .await
        .expect("Failed to check existence");
    assert!(!exists, "File should not exist after deletion");
}

#[actix_web::test]
async fn test_exists_check() {
    let backend = create_test_backend().await;
    let test_key = "test/file-exists.txt";
    let test_data = b"Exists check".to_vec();

    // Should not exist initially
    let exists = backend
        .exists(test_key)
        .await
        .expect("Failed to check existence");
    assert!(!exists, "File should not exist initially");

    // Store
    backend
        .store(test_key, test_data, None)
        .await
        .expect("Failed to store file");

    // Should exist now
    let exists = backend
        .exists(test_key)
        .await
        .expect("Failed to check existence");
    assert!(exists, "File should exist after upload");

    // Cleanup
    backend.delete(test_key).await.ok();
}

#[actix_web::test]
async fn test_large_file_upload() {
    let backend = create_test_backend().await;
    let test_key = "test/large-file.bin";
    // 5MB test file
    let test_data = vec![0u8; 5 * 1024 * 1024];

    let etag = backend
        .store(test_key, test_data.clone(), Some("application/octet-stream"))
        .await
        .expect("Failed to store large file");

    assert!(!etag.is_empty(), "ETag should not be empty for large file");

    let retrieved = backend
        .retrieve(test_key)
        .await
        .expect("Failed to retrieve large file");

    assert_eq!(
        retrieved.len(),
        test_data.len(),
        "Retrieved file size should match uploaded size"
    );

    // Cleanup
    backend.delete(test_key).await.ok();
}

#[actix_web::test]
async fn test_binary_data_integrity() {
    let backend = create_test_backend().await;
    let test_key = "test/binary-data.bin";

    // Create test data with all byte values
    let mut test_data = vec![0u8; 256];
    for i in 0..256 {
        test_data[i] = i as u8;
    }

    backend
        .store(test_key, test_data.clone(), Some("application/octet-stream"))
        .await
        .expect("Failed to store binary data");

    let retrieved = backend
        .retrieve(test_key)
        .await
        .expect("Failed to retrieve binary data");

    assert_eq!(
        retrieved, test_data,
        "Binary data should be byte-perfect after upload/download"
    );

    // Cleanup
    backend.delete(test_key).await.ok();
}

#[actix_web::test]
async fn test_concurrent_uploads() {
    let backend = Arc::new(create_test_backend().await);
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let backend_clone = Arc::clone(&backend);
            actix_web::rt::spawn(async move {
                let test_key = format!("test/concurrent-{}.txt", i);
                let test_data = format!("Concurrent file {}", i).into_bytes();

                backend_clone
                    .store(&test_key, test_data.clone(), Some("text/plain"))
                    .await
                    .expect("Failed to store concurrent file")
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    for result in results {
        assert!(result.is_ok(), "All concurrent uploads should succeed");
    }

    // Cleanup
    for i in 0..5 {
        let test_key = format!("test/concurrent-{}.txt", i);
        backend.delete(&test_key).await.ok();
    }
}
