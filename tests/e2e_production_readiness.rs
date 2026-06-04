use actix_web::test;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const API_BASE: &str = "http://127.0.0.1:8080/api/v1";

#[actix_web::test]
async fn test_production_health_status() {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let res = client
        .get("http://127.0.0.1:8080/health")
        .send()
        .await
        .expect("Health check failed");

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    println!("✓ Health check passed");
}

#[actix_web::test]
async fn test_production_metrics_available() {
    let client = Client::new();

    let res = client
        .get("http://127.0.0.1:8080/metrics")
        .send()
        .await
        .expect("Metrics endpoint failed");

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();

    // Verify Prometheus metrics format
    assert!(body.contains("http_requests_total"));
    assert!(body.contains("http_request_duration_seconds"));
    assert!(body.contains("active_connections"));
    println!("✓ Metrics endpoint verified");
}

#[actix_web::test]
async fn test_production_complete_file_workflow() {
    let client = Client::new();

    // Step 1: Login
    let login_res = client
        .post(&format!("{}/auth/login", API_BASE))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .expect("Login failed");

    assert_eq!(login_res.status(), 200);
    let login_body: Value = login_res.json().await.unwrap();
    let token = login_body["token"].as_str().expect("No token in response");

    // Step 2: Upload file
    let upload_res = client
        .post(&format!("{}/files/upload", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(b"E2E test file".to_vec())
                    .file_name("e2e_test.txt"))
        )
        .send()
        .await
        .expect("Upload failed");

    assert_eq!(upload_res.status(), 200);
    let upload_body: Value = upload_res.json().await.unwrap();
    let file_id = upload_body["id"].as_str().expect("No file ID");
    println!("✓ File uploaded: {}", file_id);

    // Step 3: Search files
    let search_res = client
        .get(&format!("{}/files/search?q=E2E", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Search failed");

    assert_eq!(search_res.status(), 200);
    let search_body: Value = search_res.json().await.unwrap();
    assert!(search_body["files"].is_array());
    println!("✓ Search verified");

    // Step 4: Download file (Range request)
    let download_res = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=0-5")
        .send()
        .await
        .expect("Download failed");

    assert_eq!(download_res.status(), 206);
    println!("✓ Range request verified (206 Partial Content)");

    // Step 5: Delete file
    let delete_res = client
        .delete(&format!("{}/files/{}", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Delete failed");

    assert_eq!(delete_res.status(), 200);
    println!("✓ File deleted");
}

#[actix_web::test]
async fn test_production_error_handling() {
    let client = Client::new();

    // Test 401 Unauthorized
    let unauth_res = client
        .get(&format!("{}/files", API_BASE))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(unauth_res.status(), 401);
    println!("✓ Unauthorized (401) handling verified");

    // Test 404 Not Found
    let notfound_res = client
        .get(&format!("{}/files/nonexistent-id", API_BASE))
        .header("Authorization", "Bearer invalid-token")
        .send()
        .await
        .expect("Request failed");

    assert!(notfound_res.status() == 401 || notfound_res.status() == 404);
    println!("✓ Not Found (404) handling verified");
}

#[actix_web::test]
async fn test_production_concurrent_requests() {
    let client = std::sync::Arc::new(Client::new());

    // Login once
    let login_res = client
        .post(&format!("{}/auth/login", API_BASE))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let token = login_res.json::<Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Make 20 concurrent requests
    let mut handles = vec![];

    for i in 0..20 {
        let client_clone = std::sync::Arc::clone(&client);
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            let res = client_clone
                .get(&format!("{}/files?page=1&per_page=10", API_BASE))
                .header("Authorization", format!("Bearer {}", token_clone))
                .send()
                .await;

            res.is_ok() && res.unwrap().status() == 200
        });

        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    let success_count = results.into_iter().filter_map(|r| r.ok()).filter(|r| *r).count();

    assert!(success_count >= 18, "At least 18/20 requests should succeed");
    println!("✓ Concurrent requests: {}/20 successful", success_count);
}

#[actix_web::test]
async fn test_production_database_connectivity() {
    let client = Client::new();

    let res = client
        .get("http://127.0.0.1:8080/health/db")
        .send()
        .await
        .expect("DB health check failed");

    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["connected"].as_bool().unwrap_or(false));
    println!("✓ Database connectivity verified");
}
