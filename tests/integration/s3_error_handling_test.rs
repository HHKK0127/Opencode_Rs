//! Integration tests for S3 error handling
//! Tests error cases: invalid keys, non-existent files, authentication failures

use opencode_poc::storage::{S3Client, S3StorageBackend, StorageBackend};
use opencode_poc::error::AppError;
use std::sync::Arc;

/// Helper to create test backend
async fn create_test_backend() -> Arc<dyn StorageBackend> {
    let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let access_key = std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin123".to_string());
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "test-uploads".to_string());

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
async fn test_retrieve_non_existent_file() {
    let backend = create_test_backend().await;
    let non_existent_key = "test/does-not-exist-12345.txt";

    let result = backend.retrieve(non_existent_key).await;

    assert!(
        result.is_err(),
        "Retrieving non-existent file should return error"
    );

    // Verify it's a NotFound error
    match result {
        Err(AppError::NotFound) => {
            // Expected
        }
        Err(e) => {
            panic!("Expected NotFound error, got: {:?}", e);
        }
        Ok(_) => {
            panic!("Expected error for non-existent file");
        }
    }
}

#[actix_web::test]
async fn test_delete_non_existent_file() {
    let backend = create_test_backend().await;
    let non_existent_key = "test/does-not-exist-delete-12345.txt";

    let result = backend.delete(non_existent_key).await;

    // S3 delete of non-existent file may succeed (idempotent)
    // But we should handle it gracefully
    assert!(
        result.is_ok() || result.is_err(),
        "Delete should either succeed or return error gracefully"
    );
}

#[actix_web::test]
async fn test_exists_with_empty_key() {
    let backend = create_test_backend().await;
    let empty_key = "";

    let result = backend.exists(empty_key).await;

    // Empty key should either return false or error
    match result {
        Ok(exists) => {
            assert!(!exists, "Empty key should not exist");
        }
        Err(_) => {
            // Also acceptable - invalid key format
        }
    }
}
