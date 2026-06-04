use actix_web::test;
use actix_web::web;
use reqwest::Client;
use serde_json::{json, Value};

#[actix_web::test]
async fn test_basic_file_upload() {
    let client = Client::new();

    // Upload a test file
    let file_content = b"test file content";
    let response = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("test.txt"))
        )
        .send()
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert!(body["id"].is_string());
    assert_eq!(body["original_name"], "test.txt");
    assert!(body["checksum"].is_string());
}

#[actix_web::test]
async fn test_get_file_metadata() {
    let client = Client::new();

    // First upload a file
    let file_content = b"test metadata content";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("metadata_test.pdf"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Retrieve metadata
    let meta_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}", file_id))
        .send()
        .await
        .unwrap();

    assert_eq!(meta_resp.status(), 200);
    let meta_body: Value = meta_resp.json().await.unwrap();

    assert_eq!(meta_body["id"], file_id);
    assert_eq!(meta_body["original_name"], "metadata_test.pdf");
    assert!(meta_body["size"].is_number());
    assert!(meta_body["mime_type"].is_string());
}

#[actix_web::test]
async fn test_download_file() {
    let client = Client::new();

    // Upload a file
    let file_content = b"downloadable content";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("download_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Download the file
    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 200);
    let downloaded_content = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded_content.as_ref(), file_content);
}

#[actix_web::test]
async fn test_delete_file() {
    let client = Client::new();

    // Upload a file
    let file_content = b"file to delete";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("delete_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Delete the file
    let delete_resp = client
        .delete(&format!("http://127.0.0.1:8080/api/v1/files/{}", file_id))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), 200);
    let delete_body: Value = delete_resp.json().await.unwrap();
    assert_eq!(delete_body["status"], "success");

    // Verify file is deleted
    let meta_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}", file_id))
        .send()
        .await
        .unwrap();

    assert_eq!(meta_resp.status(), 404);
}

#[actix_web::test]
async fn test_list_files_with_pagination() {
    let client = Client::new();

    // Upload multiple files
    for i in 0..5 {
        let content = format!("file {}", i).into_bytes();
        let _ = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(content)
                        .file_name(format!("list_test_{}.txt", i)))
            )
            .send()
            .await;
    }

    // Test pagination
    let list_resp = client
        .get("http://127.0.0.1:8080/api/v1/files?page=1&per_page=3")
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

    let init_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/init")
        .json(&json!({
            "filename": "large_file.bin",
            "total_size": 10485760,  // 10MB
            "chunk_size": 1048576    // 1MB chunks
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

    // Step 1: Initialize chunked upload
    let init_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/init")
        .json(&json!({
            "filename": "chunked_file.bin",
            "total_size": 2097152,   // 2MB
            "chunk_size": 1048576    // 1MB chunks
        }))
        .send()
        .await
        .unwrap();

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap().to_string();

    // Step 2: Upload chunks
    let chunk_data = vec![0u8; 1048576]; // 1MB of zeros

    for chunk_index in 0..2 {
        let chunk_resp = client
            .post(&format!("http://127.0.0.1:8080/api/v1/files/upload/chunk"))
            .multipart(
                reqwest::multipart::Form::new()
                    .text("session_id", session_id.clone())
                    .text("chunk_index", chunk_index.to_string())
                    .part("chunk", reqwest::multipart::Part::bytes(chunk_data.clone()))
            )
            .send()
            .await
            .unwrap();

        assert_eq!(chunk_resp.status(), 200);
        let chunk_body: Value = chunk_resp.json().await.unwrap();
        assert_eq!(chunk_body["session_id"], session_id);
        assert_eq!(chunk_body["chunk_index"], chunk_index);
    }

    // Step 3: Complete upload
    let complete_resp = client
        .post(&format!("http://127.0.0.1:8080/api/v1/files/upload/complete/{}", session_id))
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

    // Initialize chunked upload
    let init_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/init")
        .json(&json!({
            "filename": "progress_test.bin",
            "total_size": 5242880,   // 5MB
            "chunk_size": 1048576    // 1MB chunks
        }))
        .send()
        .await
        .unwrap();

    let init_body: Value = init_resp.json().await.unwrap();
    let session_id = init_body["session_id"].as_str().unwrap();

    // Upload one chunk
    let chunk_data = vec![0u8; 1048576]; // 1MB
    let _ = client
        .post("http://127.0.0.1:8080/api/v1/files/upload/chunk")
        .multipart(
            reqwest::multipart::Form::new()
                .text("session_id", session_id)
                .text("chunk_index", "0")
                .part("chunk", reqwest::multipart::Part::bytes(chunk_data.clone()))
        )
        .send()
        .await;

    // Check progress
    let progress_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/upload/progress/{}", session_id))
        .send()
        .await
        .unwrap();

    assert_eq!(progress_resp.status(), 200);
    let progress_body: Value = progress_resp.json().await.unwrap();

    assert_eq!(progress_body["total_size"], 5242880);
    assert_eq!(progress_body["uploaded_size"], 1048576);
    assert!(progress_body["progress_percent"].is_number());
    assert_eq!(progress_body["remaining_bytes"], 4194304);
}

#[actix_web::test]
async fn test_file_size_limit() {
    let client = Client::new();

    // Try to upload file exceeding size limit
    let large_content = vec![0u8; 150 * 1024 * 1024]; // 150MB

    let resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(large_content)
                    .file_name("too_large.bin"))
        )
        .send()
        .await
        .unwrap();

    // Should get 400 error for exceeding limit
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_nonexistent_file() {
    let client = Client::new();

    // Try to access non-existent file
    let resp = client
        .get("http://127.0.0.1:8080/api/v1/files/nonexistent-id")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_filename_sanitization() {
    let client = Client::new();

    let file_content = b"sanitized content";
    let resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("test@#$%^&()file.txt"))
        )
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    // Original name should be sanitized
    let original_name = body["original_name"].as_str().unwrap();
    assert!(!original_name.contains("@"));
    assert!(!original_name.contains("#"));
}

#[actix_web::test]
async fn test_checksum_verification() {
    let client = Client::new();

    let file_content = b"test checksum content";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("checksum_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let body: Value = upload_resp.json().await.unwrap();
    let checksum = body["checksum"].as_str().unwrap();

    // Verify checksum is SHA-256 (64 hex characters)
    assert_eq!(checksum.len(), 64);
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
}
