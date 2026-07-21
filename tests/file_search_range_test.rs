#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    clippy::all
)]

use reqwest::Client;
use serde_json::Value;

const API_BASE: &str = "http://127.0.0.1:8080/api/v1";

async fn get_token(client: &Client) -> String {
    let resp = client
        .post(&format!("{}/auth/login", API_BASE))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .expect("Login request failed");
    let body: Value = resp.json().await.unwrap();
    body["token"]
        .as_str()
        .expect("No token in login response")
        .to_string()
}

async fn upload_file(client: &Client, token: &str, filename: &str, content: Vec<u8>) -> String {
    let resp = client
        .post(&format!("{}/files/upload", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(content).file_name(filename.to_string()),
        ))
        .send()
        .await
        .expect("Upload failed");
    let body: Value = resp.json().await.unwrap();
    body["id"].as_str().expect("No file ID").to_string()
}

#[actix_web::test]
async fn test_range_request_single_byte_range() {
    let client = Client::new();
    let token = get_token(&client).await;

    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz".to_vec();
    let file_id = upload_file(&client, &token, "range_test.txt", file_content).await;

    let download_resp = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=0-9")
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 206);
    assert!(download_resp.headers().get("Content-Range").is_some());
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), b"0123456789");
}

#[actix_web::test]
async fn test_range_request_suffix_range() {
    let client = Client::new();
    let token = get_token(&client).await;

    // 36-byte file: "0123456789abcdefghijklmnopqrstuvwxyz"
    // bytes=-10 returns last 10 bytes: "qrstuvwxyz"
    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz".to_vec();
    let file_id = upload_file(&client, &token, "suffix_range_test.txt", file_content).await;

    let download_resp = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=-10")
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 206);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.len(), 10);
    assert_eq!(downloaded.as_ref(), b"qrstuvwxyz");
}

#[actix_web::test]
async fn test_range_request_from_offset() {
    let client = Client::new();
    let token = get_token(&client).await;

    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz".to_vec();
    let file_id = upload_file(&client, &token, "offset_range_test.txt", file_content).await;

    let download_resp = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=20-")
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 206);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), b"klmnopqrstuvwxyz");
}

#[actix_web::test]
async fn test_range_request_content_range_header() {
    let client = Client::new();
    let token = get_token(&client).await;

    let file_content = b"0123456789".to_vec();
    let file_id = upload_file(&client, &token, "content_range_test.txt", file_content).await;

    let download_resp = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Range", "bytes=0-4")
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 206);
    let content_range = download_resp.headers().get("Content-Range").unwrap();
    assert_eq!(content_range.to_str().unwrap(), "bytes 0-4/10");
}

#[actix_web::test]
async fn test_search_files_by_keyword() {
    let client = Client::new();
    let token = get_token(&client).await;

    for filename in &["report.pdf", "document.pdf", "image.jpg"] {
        upload_file(&client, &token, filename, b"test content".to_vec()).await;
    }

    let search_resp = client
        .get(&format!("{}/files/search?q=report", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert!(body["total"].is_number());
    assert!(body["page"].is_number());
}

#[actix_web::test]
async fn test_search_files_by_mime_type() {
    let client = Client::new();
    let token = get_token(&client).await;

    let search_resp = client
        .get(&format!(
            "{}/files/search?mime_type=application/pdf",
            API_BASE
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert!(body["total"].is_number());
}

#[actix_web::test]
async fn test_search_files_by_size_range() {
    let client = Client::new();
    let token = get_token(&client).await;

    let search_resp = client
        .get(&format!(
            "{}/files/search?size_min=1&size_max=10000000",
            API_BASE
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert!(body["total"].is_number());
}

#[actix_web::test]
async fn test_search_files_with_sorting() {
    let client = Client::new();
    let token = get_token(&client).await;

    let search_resp = client
        .get(&format!("{}/files/search?sort=size&order=desc", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
}

#[actix_web::test]
async fn test_file_statistics() {
    let client = Client::new();
    let token = get_token(&client).await;

    let stats_resp = client
        .get(&format!("{}/files/stats", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(stats_resp.status(), 200);
    let body: Value = stats_resp.json().await.unwrap();
    assert!(body["total_files"].is_number());
    assert!(body["total_size_bytes"].is_number());
    assert!(body["average_size_bytes"].is_number());
}
