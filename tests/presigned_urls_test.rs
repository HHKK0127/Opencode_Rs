use opencode_poc::app_state::AppState;
use opencode_poc::config::Settings;
use opencode_poc::storage::s3_client::S3Client;
use sqlx::sqlite::SqlitePool;

#[tokio::test]
async fn test_presigned_put_url_generation() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    let app_state = AppState::new(settings, pool, s3_client);

    // Test: Generate presigned PUT URL
    let result = app_state.s3_client
        .generate_presigned_put_url(
            "test/file.txt",
            std::time::Duration::from_secs(300),
            Some("text/plain"),
        )
        .await;

    match result {
        Ok(url) => {
            // Verify URL structure
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

#[tokio::test]
async fn test_presigned_get_url_generation() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    let app_state = AppState::new(settings, pool, s3_client);

    // Test: Generate presigned GET URL
    let result = app_state.s3_client
        .generate_presigned_get_url(
            "test/file.txt",
            std::time::Duration::from_secs(3600),
        )
        .await;

    match result {
        Ok(url) => {
            // Verify URL structure
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

#[tokio::test]
async fn test_presigned_url_expiry_validation() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    // Test: PUT URL with max valid expiry (24 hours)
    let result_valid = s3_client
        .generate_presigned_put_url(
            "test/file.txt",
            std::time::Duration::from_secs(86400),
            None,
        )
        .await;

    assert!(result_valid.is_ok(), "24-hour TTL should be valid");
    println!("✅ Presigned URL with 24h TTL validated");

    // Test: PUT URL with minimal expiry (1 second)
    let result_min = s3_client
        .generate_presigned_put_url(
            "test/file.txt",
            std::time::Duration::from_secs(1),
            None,
        )
        .await;

    assert!(result_min.is_ok(), "1-second TTL should be valid");
    println!("✅ Presigned URL with 1s TTL validated");
}

#[tokio::test]
async fn test_presigned_put_url_signature() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    // Generate URL
    let url = s3_client
        .generate_presigned_put_url(
            "test/file.txt",
            std::time::Duration::from_secs(300),
            Some("text/plain"),
        )
        .await
        .expect("Failed to generate URL");

    // Verify signature components
    assert!(url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"), "Missing AWS4 algorithm");
    assert!(url.contains("X-Amz-Credential"), "Missing credentials");
    assert!(url.contains("X-Amz-Date"), "Missing date");
    assert!(url.contains("X-Amz-Expires=300"), "Incorrect expiry");
    assert!(url.contains("X-Amz-Signature"), "Missing signature");
    assert!(url.contains("X-Amz-SignedHeaders"), "Missing signed headers");

    println!("✅ Presigned PUT URL signature verified");
}

#[tokio::test]
async fn test_presigned_get_url_signature() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    // Generate URL
    let url = s3_client
        .generate_presigned_get_url(
            "test/file.txt",
            std::time::Duration::from_secs(3600),
        )
        .await
        .expect("Failed to generate URL");

    // Verify signature components
    assert!(url.contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"), "Missing AWS4 algorithm");
    assert!(url.contains("X-Amz-Credential"), "Missing credentials");
    assert!(url.contains("X-Amz-Date"), "Missing date");
    assert!(url.contains("X-Amz-Expires=3600"), "Incorrect expiry");
    assert!(url.contains("X-Amz-Signature"), "Missing signature");
    assert!(url.contains("X-Amz-SignedHeaders"), "Missing signed headers");

    println!("✅ Presigned GET URL signature verified");
}

#[tokio::test]
async fn test_presigned_urls_with_special_characters() {
    // Setup
    let settings = Settings::default();
    let database_url = "sqlite://";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to create in-memory pool");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    // Test filenames with special characters
    let test_files = vec![
        "test/file-with-dashes.txt",
        "test/file_with_underscores.txt",
        "test/file.multiple.dots.txt",
        "test/文件.txt", // Unicode characters
    ];

    for filename in test_files {
        let result = s3_client
            .generate_presigned_put_url(
                filename,
                std::time::Duration::from_secs(300),
                None,
            )
            .await;

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
