// Wave 3: Presigned URL Generation Tests (PostgreSQL-ready)
//
// Migrated from tests/legacy/presigned_urls_test.rs
// - Removed AppState wrapping (current AppState has no s3_client field)
// - Tests S3Client directly with Settings

use opencode_poc::config::Settings;
use opencode_poc::storage::s3_client::S3Client;
use std::time::Duration;

// Reuse S3Client if available, otherwise skip gracefully
async fn get_s3_client() -> Option<S3Client> {
    let settings = Settings::default();
    S3Client::new(&settings).await.ok()
}

/// Test 1: Generate presigned PUT URL
#[actix_rt::test]
async fn test_presigned_put_url_generation() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let result = client.generate_presigned_put_url("test/file.txt", Duration::from_secs(300), Some("text/plain")).await;

    match result {
        Ok(url) => {
            assert!(url.contains("http://localhost:9000") || url.contains("minio"));
            assert!(url.contains("test/file.txt"));
            assert!(url.contains("X-Amz-Signature"));
            println!("✅ Presigned PUT URL generated: {}", url);
        }
        Err(e) => {
            panic!("Failed to generate presigned PUT URL: {:?}", e);
        }
    }
}

/// Test 2: Generate presigned GET URL
#[actix_rt::test]
async fn test_presigned_get_url_generation() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let result = client.generate_presigned_get_url("test/file.txt", Duration::from_secs(3600)).await;

    match result {
        Ok(url) => {
            assert!(url.contains("http://localhost:9000") || url.contains("minio"));
            assert!(url.contains("test/file.txt"));
            assert!(url.contains("X-Amz-Signature"));
            println!("✅ Presigned GET URL generated: {}", url);
        }
        Err(e) => {
            panic!("Failed to generate presigned GET URL: {:?}", e);
        }
    }
}

/// Test 3: Validate expiry range (1 second to 24 hours)
#[actix_rt::test]
async fn test_presigned_url_expiry_validation() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    // 24-hour TTL should be valid
    let result_valid = client.generate_presigned_put_url("test/file.txt", Duration::from_secs(86400), None).await;
    assert!(result_valid.is_ok(), "24-hour TTL should be valid");
    println!("✅ Presigned URL with 24h TTL validated");

    // 1-second TTL should be valid
    let result_min = client.generate_presigned_put_url("test/file.txt", Duration::from_secs(1), None).await;
    assert!(result_min.is_ok(), "1-second TTL should be valid");
    println!("✅ Presigned URL with 1s TTL validated");
}

/// Test 4: Verify PUT URL signature components
#[actix_rt::test]
async fn test_presigned_put_url_signature() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let url = client.generate_presigned_put_url("test/file.txt", Duration::from_secs(300), Some("text/plain")).await.expect("Failed to generate URL");

    assert!(url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"), "Missing AWS4 algorithm");
    assert!(url.contains("X-Amz-Credential"), "Missing credentials");
    assert!(url.contains("X-Amz-Date"), "Missing date");
    assert!(url.contains("X-Amz-Expires=300"), "Incorrect expiry");
    assert!(url.contains("X-Amz-Signature"), "Missing signature");
    assert!(url.contains("X-Amz-SignedHeaders"), "Missing signed headers");

    println!("✅ Presigned PUT URL signature verified");
}

/// Test 5: Verify GET URL signature components
#[actix_rt::test]
async fn test_presigned_get_url_signature() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let url = client.generate_presigned_get_url("test/file.txt", Duration::from_secs(3600)).await.expect("Failed to generate URL");

    assert!(url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"), "Missing AWS4 algorithm");
    assert!(url.contains("X-Amz-Credential"), "Missing credentials");
    assert!(url.contains("X-Amz-Date"), "Missing date");
    assert!(url.contains("X-Amz-Expires=3600"), "Incorrect expiry");
    assert!(url.contains("X-Amz-Signature"), "Missing signature");
    assert!(url.contains("X-Amz-SignedHeaders"), "Missing signed headers");

    println!("✅ Presigned GET URL signature verified");
}

/// Test 6: Special characters in filenames
#[actix_rt::test]
async fn test_presigned_urls_with_special_characters() {
    let client = match get_s3_client().await {
        Some(c) => c,
        None => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let test_files = vec![
        "test/file-with-dashes.txt",
        "test/file_with_underscores.txt",
        "test/file.multiple.dots.txt",
        "test/文件.txt", // Unicode characters
    ];

    for filename in test_files {
        let result = client.generate_presigned_put_url(filename, Duration::from_secs(300), None).await;

        match result {
            Ok(url) => {
                assert!(url.contains("X-Amz-Signature"));
                println!("✅ Generated URL for: {}", filename);
            }
            Err(e) => {
                panic!("Failed for filename '{}': {:?}", filename, e);
            }
        }
    }
}
