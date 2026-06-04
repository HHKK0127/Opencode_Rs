use reqwest::Client;
use serde_json::Value;

#[actix_web::test]
async fn test_range_request_single_byte_range() {
    let client = Client::new();

    // Upload a test file
    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("range_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Request specific byte range (0-9)
    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
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

    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("suffix_range_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Request last 10 bytes
    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
        .header("Range", "bytes=-10")
        .send()
        .await
        .unwrap();

    assert_eq!(download_resp.status(), 206);
    let downloaded = download_resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), b"wxyz");  // Last 10 chars, but file is 36 bytes
}

#[actix_web::test]
async fn test_range_request_from_offset() {
    let client = Client::new();

    let file_content = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("offset_range_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    // Request from byte 20 to end
    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
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

    let file_content = b"0123456789";
    let upload_resp = client
        .post("http://127.0.0.1:8080/api/v1/files/upload")
        .multipart(
            reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.to_vec())
                    .file_name("content_range_test.txt"))
        )
        .send()
        .await
        .unwrap();

    let upload_body: Value = upload_resp.json().await.unwrap();
    let file_id = upload_body["id"].as_str().unwrap();

    let download_resp = client
        .get(&format!("http://127.0.0.1:8080/api/v1/files/{}/download", file_id))
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

    // Upload test files
    for filename in &["report.pdf", "document.pdf", "image.jpg"] {
        let _ = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .multipart(
                reqwest::multipart::Form::new()
                    .part("file", reqwest::multipart::Part::bytes(b"test content".to_vec())
                        .file_name(*filename))
            )
            .send()
            .await;
    }

    // Search by keyword
    let search_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/search?q=report")
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert!(body["pagination"].is_object());
    assert_eq!(body["query_summary"]["filters_applied"], 1);
}

#[actix_web::test]
async fn test_search_files_by_mime_type() {
    let client = Client::new();

    // Search PDF files
    let search_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/search?mime_type=application/pdf")
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert_eq!(body["query_summary"]["filters_applied"], 1);
}

#[actix_web::test]
async fn test_search_files_by_size_range() {
    let client = Client::new();

    // Search files between 1KB and 10MB
    let search_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/search?size_min=1000&size_max=10000000")
        .send()
        .await
        .unwrap();

    assert_eq!(search_resp.status(), 200);
    let body: Value = search_resp.json().await.unwrap();
    assert!(body["files"].is_array());
    assert_eq!(body["query_summary"]["filters_applied"], 2);
}

#[actix_web::test]
async fn test_search_files_with_sorting() {
    let client = Client::new();

    // Search with sort by size descending
    let search_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/search?sort=size&order=desc")
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

    // Get file statistics
    let stats_resp = client
        .get("http://127.0.0.1:8080/api/v1/files/stats")
        .send()
        .await
        .unwrap();

    assert_eq!(stats_resp.status(), 200);
    let body: Value = stats_resp.json().await.unwrap();
    assert!(body["total_files"].is_number());
    assert!(body["total_size_bytes"].is_number());
    assert!(body["average_size_bytes"].is_number());
    assert!(body["unique_mime_types"].is_string());
}
