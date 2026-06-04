use actix_web::test;
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;
use std::time::Duration;

#[actix_web::test]
async fn test_single_file_upload_performance() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Test single upload performance
    let file_content = b"test file content for performance measurement";

    let start = Instant::now();
    let response = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("perf_test.txt"))
        )
        .send()
        .await;

    let elapsed = start.elapsed();

    assert!(response.is_ok());
    assert!(elapsed < Duration::from_millis(500),
            "Single upload took {:?}, expected <500ms", elapsed);

    eprintln!("✓ Single upload: {:?}", elapsed);
}

#[actix_web::test]
async fn test_bulk_file_upload_throughput() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Upload 20 files sequentially and measure throughput
    let file_content = b"test file content";
    let start = Instant::now();

    for i in 0..20 {
        let filename = format!("bulk_test_{}.txt", i);
        let _response = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                        .file_name(filename))
            )
            .send()
            .await;
    }

    let elapsed = start.elapsed();
    let throughput = 20.0 / elapsed.as_secs_f64();

    eprintln!("✓ Bulk upload: 20 files in {:?} ({:.2} uploads/sec)", elapsed, throughput);
    assert!(throughput > 5.0, "Expected >5 uploads/sec, got {:.2}", throughput);
}

#[actix_web::test]
async fn test_search_performance_with_filters() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // First, upload some test files with different sizes
    for i in 0..10 {
        let size = (i + 1) * 1000; // 1KB, 2KB, ... 10KB
        let content = vec![0u8; size];
        let filename = format!("search_test_{}.bin", i);
        let _response = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(content)
                        .file_name(filename))
            )
            .send()
            .await;
    }

    // Now test search with multiple filters
    let start = Instant::now();
    let search_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/search?q=search&size_min=1000&size_max=100000&sort=size&order=desc")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(search_resp.status(), 200);
    assert!(elapsed < Duration::from_millis(150),
            "Search took {:?}, expected <150ms", elapsed);

    eprintln!("✓ Search with filters: {:?}", elapsed);
}

#[actix_web::test]
async fn test_range_request_performance() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Upload a test file
    let file_content = vec![0u8; 100_000]; // 100KB
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.clone())
                    .file_name("range_perf_test.bin"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Test range request performance
    let start = Instant::now();
    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=0-9999")
        .send()
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(download_resp.status(), 206);
    assert!(elapsed < Duration::from_millis(100),
            "Range request took {:?}, expected <100ms", elapsed);

    eprintln!("✓ Range request: {:?}", elapsed);
}

#[actix_web::test]
async fn test_chunked_upload_performance() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Initialize chunked upload
    let init_start = Instant::now();
    let init_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/init")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "filename": "chunked_perf_test.bin",
            "total_size": 2_097_152,  // 2MB
            "chunk_size": 1_048_576   // 1MB chunks
        }))
        .send()
        .await
        .unwrap();

    let init_elapsed = init_start.elapsed();
    assert!(init_elapsed < Duration::from_millis(100), "Init took too long: {:?}", init_elapsed);

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap().to_string();

    // Upload 2 chunks
    let chunk_data = vec![0u8; 1_048_576]; // 1MB
    let chunk_start = Instant::now();

    for i in 0..2 {
        let _chunk_resp = client
            .post("http://127.0.0.1:8080/api/v1/files/upload/chunk")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new()
                    .text("session_id", session_id.clone())
                    .text("chunk_index", i.to_string())
                    .part("chunk", reqwest::multipart::Part::bytes(chunk_data.clone()))
            )
            .send()
            .await;
    }

    let chunk_elapsed = chunk_start.elapsed();
    eprintln!("✓ Chunked upload: init {:?}, 2 chunks {:?}", init_elapsed, chunk_elapsed);

    // Complete upload
    let complete_start = Instant::now();
    let _complete_resp = client
        .post(&format!("http://127.0.0.1:8080/api/v1/files/upload/complete/{}", session_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    let complete_elapsed = complete_start.elapsed();
    assert!(complete_elapsed < Duration::from_millis(200), "Complete took too long: {:?}", complete_elapsed);
}

#[actix_web::test]
async fn test_progress_tracking_performance() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Initialize chunked upload
    let init_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/init")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "filename": "progress_perf_test.bin",
            "total_size": 5_242_880,  // 5MB
            "chunk_size": 1_048_576   // 1MB chunks
        }))
        .send()
        .await
        .unwrap();

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap().to_string();

    // Upload one chunk
    let chunk_data = vec![0u8; 1_048_576];
    let _chunk_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/chunk")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new()
                .text("session_id", session_id.clone())
                .text("chunk_index", "0")
                .part("chunk", reqwest::multipart::Part::bytes(chunk_data))
        )
        .send()
        .await;

    // Query progress 10 times and measure average
    let mut total_time = Duration::ZERO;
    for _ in 0..10 {
        let start = Instant::now();
        let _progress_resp = client
            .get(&format!("http://127.0.0.1:8080/api/v1/files/upload/progress/{}", session_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;
        total_time += start.elapsed();
    }

    let avg_time = total_time / 10;
    assert!(avg_time < Duration::from_millis(10),
            "Progress query avg {:?}, expected <10ms", avg_time);

    eprintln!("✓ Progress tracking: avg {:?} (10 queries)", avg_time);
}

#[actix_web::test]
async fn test_file_listing_pagination_performance() {
    let client = Client::new();

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();

    // Upload 50 files
    for i in 0..50 {
        let filename = format!("list_test_{}.txt", i);
        let _response = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(b"test".to_vec())
                        .file_name(filename))
            )
            .send()
            .await;
    }

    // Test listing with pagination
    let start = Instant::now();
    let list_resp = client
        .get("http://127.0.0.1:8080/api/v1/files?page=1&per_page=20")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(list_resp.status(), 200);
    assert!(elapsed < Duration::from_millis(200),
            "Listing took {:?}, expected <200ms", elapsed);

    eprintln!("✓ Pagination listing: {:?}", elapsed);
}
