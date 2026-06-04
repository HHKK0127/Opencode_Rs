//! Integration tests for S3 health check operations

use opencode_poc::storage::{S3Client, S3StorageBackend, StorageBackend};
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
async fn test_storage_backend_health_check() {
    let backend = create_test_backend().await;

    let result = backend.health_check().await;

    assert!(
        result.is_ok(),
        "Health check should succeed when S3 is running"
    );
}

#[actix_web::test]
async fn test_health_check_connection() {
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

    // Attempt to create client (which includes connection test)
    let client_result = S3Client::new(&settings).await;

    assert!(
        client_result.is_ok(),
        "S3Client should be created when S3 is running"
    );
}

#[actix_web::test]
async fn test_multiple_health_checks() {
    let backend = create_test_backend().await;

    // Run health check multiple times
    for _ in 0..5 {
        let result = backend.health_check().await;
        assert!(result.is_ok(), "Repeated health checks should all succeed");
    }
}
