//! Integration tests for S3 multipart upload operations
//! Tests multipart upload flow: init → chunk upload → complete

use opencode_poc::storage::S3Client;
use opencode_poc::config::Settings;

/// Helper to create test S3Client
async fn create_test_s3_client() -> S3Client {
    let endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let access_key = std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin123".to_string());
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "test-uploads".to_string());

    let settings = Settings {
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

    S3Client::new(&settings)
        .await
        .expect("Failed to create S3Client for testing")
}

#[actix_web::test]
async fn test_multipart_upload_init() {
    let client = create_test_s3_client().await;
    let test_key = "test/multipart-init.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    assert!(!upload_id.is_empty(), "Upload ID should not be empty");
}

#[actix_web::test]
async fn test_multipart_upload_single_part() {
    let client = create_test_s3_client().await;
    let test_key = "test/multipart-single-part.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    let test_data = b"Single part data".to_vec();

    let part = client
        .upload_part(test_key, &upload_id, 1, test_data)
        .await
        .expect("Failed to upload part");

    assert_eq!(part.part_number, 1, "Part number should match");
    assert!(!part.e_tag.is_empty(), "ETag should not be empty");

    // Complete
    let parts = vec![part];
    let result = client
        .complete_multipart_upload(test_key, &upload_id, parts)
        .await
        .expect("Failed to complete multipart upload");

    assert!(!result.is_empty(), "Result ETag should not be empty");
}

#[actix_web::test]
async fn test_multipart_upload_multiple_parts() {
    let client = create_test_s3_client().await;
    let test_key = "test/multipart-multi-part.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    // Upload 3 parts
    let mut parts = vec![];
    for i in 1..=3 {
        let test_data = format!("Part {} data", i).into_bytes();
        let part = client
            .upload_part(test_key, &upload_id, i, test_data)
            .await
            .expect("Failed to upload part");
        parts.push(part);
    }

    assert_eq!(parts.len(), 3, "Should have 3 parts");

    // Complete
    let result = client
        .complete_multipart_upload(test_key, &upload_id, parts)
        .await
        .expect("Failed to complete multipart upload");

    assert!(!result.is_empty(), "Result ETag should not be empty");
}

#[actix_web::test]
async fn test_multipart_upload_large_file() {
    let client = create_test_s3_client().await;
    let test_key = "test/multipart-large-file.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    // Upload 5 parts of 1MB each = 5MB total
    let part_size = 1024 * 1024; // 1MB
    let mut parts = vec![];

    for i in 1..=5 {
        let test_data = vec![i as u8; part_size];
        let part = client
            .upload_part(test_key, &upload_id, i, test_data)
            .await
            .expect("Failed to upload part");
        parts.push(part);
    }

    assert_eq!(parts.len(), 5, "Should have 5 parts");

    // Complete
    let result = client
        .complete_multipart_upload(test_key, &upload_id, parts)
        .await
        .expect("Failed to complete multipart upload");

    assert!(!result.is_empty(), "Result ETag should not be empty");

    // Verify final size
    let retrieved = client
        .download_object(test_key)
        .await
        .expect("Failed to retrieve multipart file");

    assert_eq!(
        retrieved.len(),
        part_size * 5,
        "File size should be 5MB"
    );

    // Cleanup
    client.delete_object(test_key).await.ok();
}

#[actix_web::test]
async fn test_multipart_upload_with_different_sizes() {
    let client = create_test_s3_client().await;
    let test_key = "test/multipart-varying-sizes.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    // Upload parts with different sizes
    let sizes = vec![512 * 1024, 1024 * 1024, 256 * 1024, 2048 * 1024]; // 512KB, 1MB, 256KB, 2MB
    let mut parts = vec![];
    let mut total_size = 0;

    for (i, size) in sizes.iter().enumerate() {
        let test_data = vec![(i + 1) as u8; *size];
        total_size += size;

        let part = client
            .upload_part(test_key, &upload_id, (i + 1) as i32, test_data)
            .await
            .expect("Failed to upload part");
        parts.push(part);
    }

    // Complete
    let result = client
        .complete_multipart_upload(test_key, &upload_id, parts)
        .await
        .expect("Failed to complete multipart upload");

    assert!(!result.is_empty(), "Result ETag should not be empty");

    // Verify total size
    let retrieved = client
        .download_object(test_key)
        .await
        .expect("Failed to retrieve multipart file");

    assert_eq!(
        retrieved.len(),
        total_size,
        "File size should match total of all parts"
    );

    // Cleanup
    client.delete_object(test_key).await.ok();
}

#[actix_web::test]
async fn test_multipart_upload_concurrent_parts() {
    let client = std::sync::Arc::new(create_test_s3_client().await);
    let test_key = "test/multipart-concurrent-parts.bin";

    let upload_id = client
        .initiate_multipart_upload(test_key)
        .await
        .expect("Failed to initiate multipart upload");

    let upload_id = std::sync::Arc::new(upload_id);

    // Upload 5 parts concurrently
    let handles: Vec<_> = (1..=5)
        .map(|i| {
            let client_clone = std::sync::Arc::clone(&client);
            let upload_id_clone = std::sync::Arc::clone(&upload_id);
            let test_key = test_key.to_string();

            actix_web::rt::spawn(async move {
                let test_data = format!("Concurrent part {}", i).as_bytes().to_vec();
                client_clone
                    .upload_part(&test_key, &upload_id_clone, i, test_data)
                    .await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    let mut parts = vec![];
    for result in results {
        let part = result
            .expect("Task panicked")
            .expect("Upload part failed");
        parts.push(part);
    }

    assert_eq!(parts.len(), 5, "Should have 5 parts");

    // Complete
    let result = client
        .complete_multipart_upload(test_key, &upload_id, parts)
        .await
        .expect("Failed to complete multipart upload");

    assert!(!result.is_empty(), "Result ETag should not be empty");

    // Cleanup
    client.delete_object(test_key).await.ok();
}
