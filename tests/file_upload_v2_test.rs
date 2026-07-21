#![cfg(feature = "postgres")]

use reqwest::Client;
use serde_json::{json, Value};

const API_BASE: &str = "http://127.0.0.1:8080/api/v1";

async fn get_token(client: &Client) -> String {
    let resp = client
        .post(&format!("{}/auth/login", API_BASE))
        .json(&json!({"username": "testuser", "password": "testpassword"}))
        .send()
        .await
        .expect("Login failed");
    let body: Value = resp.json().await.unwrap();
    body["token"].as_str().expect("No token").to_string()
}

async fn upload(client: &Client, token: &str, filename: &str, content: Vec<u8>) -> Value {
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
    resp.json().await.unwrap()
}

#[actix_web::test]
async fn test_basic_file_upload() {
    let client = Client::new();
    let token = get_token(&client).await;

    let body = upload(&client, &token, "test.txt", b"test file content".to_vec()).await;
    assert!(body["id"].is_string());
    assert!(body["filename"].is_string());
    assert!(body["checksum"].is_string());
}

#[actix_web::test]
async fn test_get_file_metadata() {
    let client = Client::new();
    let token = get_token(&client).await;

    let upload_body = upload(
        &client,
        &token,
        "metadata_test.pdf",
        b"test metadata content".to_vec(),
    )
    .await;
    let file_id = upload_body["id"].as_str().unwrap();

    let meta_resp = client
        .get(&format!("{}/files/{}", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(meta_resp.status(), 200);
    let meta_body: Value = meta_resp.json().await.unwrap();
    assert_eq!(meta_body["id"], file_id);
    assert!(meta_body["size"].is_number());
    assert!(meta_body["mime_type"].is_string());
}

#[actix_web::test]
async fn test_download_file() {
    let client = Client::new();
    let token = get_token(&client).await;

    let file_content = b"downloadable content";
    let upload_body = upload(&client, &token, "download_test.txt", file_content.to_vec()).await;
    let file_id = upload_body["id"].as_str().unwrap();

    let download_resp = client
        .get(&format!("{}/files/{}/download", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 200);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), file_content);
}

#[actix_web::test]
async fn test_delete_file() {
    let client = Client::new();
    let token = get_token(&client).await;

    let upload_body = upload(
        &client,
        &token,
        "delete_test.txt",
        b"file to delete".to_vec(),
    )
    .await;
    let file_id = upload_body["id"].as_str().unwrap();

    let delete_resp = client
        .delete(&format!("{}/files/{}", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), 200);
    let delete_body: Value = delete_resp.json().await.unwrap();
    assert_eq!(delete_body["status"], "success");

    // Verify file is deleted
    let meta_resp = client
        .get(&format!("{}/files/{}", API_BASE, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(meta_resp.status(), 404);
}

#[actix_web::test]
async fn test_list_files_with_pagination() {
    let client = Client::new();
    let token = get_token(&client).await;

    for i in 0..5 {
        upload(
            &client,
            &token,
            &format!("list_test_{}.txt", i),
            format!("file {}", i).into_bytes(),
        )
        .await;
    }

    let list_resp = client
        .get(&format!("{}/files?page=1&per_page=3", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(list_resp.status(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    assert!(list_body["files"].is_array());
    assert!(list_body["pagination"].is_object());
    assert_eq!(list_body["pagination"]["page"], 1);
    assert_eq!(list_body["pagination"]["per_page"], 3);
}

#[actix_web::test]
async fn test_init_chunked_upload() {
    let client = Client::new();
    let token = get_token(&client).await;

    let init_resp = client
        .post(&format!("{}/files/upload/init", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "filename": "large_file.bin",
            "total_size": 10485760,
            "chunk_size": 1048576
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(init_resp.status(), 200);
    let body: Value = init_resp.json().await.unwrap();
    assert!(body["session_id"].is_string());
    assert_eq!(body["chunk_size"], 1048576);
}

#[actix_web::test]
async fn test_chunked_upload_flow() {
    let client = Client::new();
    let token = get_token(&client).await;

    let init_resp = client
        .post(&format!("{}/files/upload/init", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "filename": "chunked_file.bin",
            "total_size": 2097152,
            "chunk_size": 1048576
        }))
        .send()
        .await
        .unwrap();

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap().to_string();

    let chunk_data = vec![0u8; 1048576];
    for chunk_index in 0..2u32 {
        let chunk_resp = client
            .post(&format!("{}/files/upload/chunk/{}", API_BASE, session_id))
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new()
                    .text("chunk_index", chunk_index.to_string())
                    .part("chunk", reqwest::multipart::Part::bytes(chunk_data.clone())),
            )
            .send()
            .await
            .unwrap();

        assert_eq!(chunk_resp.status(), 200);
    }

    let complete_resp = client
        .post(&format!(
            "{}/files/upload/complete/{}",
            API_BASE, session_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(complete_resp.status(), 200);
    let complete_body: Value = complete_resp.json().await.unwrap();
    assert!(complete_body["file_id"].is_string());
    assert_eq!(complete_body["size"], 2097152);
}

#[actix_web::test]
async fn test_get_upload_progress() {
    let client = Client::new();
    let token = get_token(&client).await;

    let init_resp = client
        .post(&format!("{}/files/upload/init", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "filename": "progress_test.bin",
            "total_size": 5242880,
            "chunk_size": 1048576
        }))
        .send()
        .await
        .unwrap();

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap().to_string();

    let chunk_data = vec![0u8; 1048576];
    let _ = client
        .post(&format!("{}/files/upload/chunk/{}", API_BASE, session_id))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new()
                .text("chunk_index", "0")
                .part("chunk", reqwest::multipart::Part::bytes(chunk_data)),
        )
        .send()
        .await;

    let progress_resp = client
        .get(&format!(
            "{}/files/upload/progress/{}",
            API_BASE, session_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(progress_resp.status(), 200);
    let body: Value = progress_resp.json().await.unwrap();
    assert_eq!(body["total_size"], 5242880);
    assert_eq!(body["uploaded_size"], 1048576);
    assert!(body["progress_percent"].is_number());
    assert_eq!(body["remaining_bytes"], 4194304);
}

#[actix_web::test]
async fn test_file_size_limit() {
    let client = Client::new();
    let token = get_token(&client).await;

    // 150MB exceeds the 100MB server limit; server may close connection or return 400
    let large_content = vec![0u8; 150 * 1024 * 1024];

    let result = client
        .post(&format!("{}/files/upload", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(large_content).file_name("too_large.bin"),
        ))
        .send()
        .await;

    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            assert!(
                status == 400 || status == 413,
                "Expected 400 or 413 for oversized file, got {}",
                status
            );
        }
        Err(_) => {} // Server may abort connection for oversized payloads
    }
}

#[actix_web::test]
async fn test_nonexistent_file() {
    let client = Client::new();
    let token = get_token(&client).await;

    let resp = client
        .get(&format!(
            "{}/files/nonexistent-file-id-that-does-not-exist",
            API_BASE
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_filename_sanitization() {
    let client = Client::new();
    let token = get_token(&client).await;

    let resp = client
        .post(&format!("{}/files/upload", API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(
            reqwest::multipart::Form::new().part(
                "file",
                reqwest::multipart::Part::bytes(b"sanitized content".to_vec())
                    .file_name("test@#$%^&()file.txt"),
            ),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let filename = body["filename"].as_str().unwrap();
    assert!(!filename.contains("@"));
    assert!(!filename.contains("#"));
}

#[actix_web::test]
async fn test_checksum_verification() {
    let client = Client::new();
    let token = get_token(&client).await;

    let body = upload(
        &client,
        &token,
        "checksum_test.txt",
        b"test checksum content".to_vec(),
    )
    .await;
    let checksum = body["checksum"].as_str().unwrap();

    // SHA-256 is 64 hex characters
    assert_eq!(checksum.len(), 64);
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
}
