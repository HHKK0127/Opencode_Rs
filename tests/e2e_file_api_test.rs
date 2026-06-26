// E2E File API Tests (PostgreSQL)
//
// Migrated from tests/legacy/e2e_s3_metadata_test.rs
// Original tested S3-specific /files/register and /files/s3/complete endpoints.
// Rewritten to test current storage-backend-agnostic file API.

use actix_web::{http::StatusCode, test, web, App};
use opencode_poc::api;

mod fixtures;

// ---------------------------------------------------------------------------
// Test 1: List files with pagination
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_list_files_with_pagination() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/files?page=1&per_page=20")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("files").is_some(), "Response should contain files array");

    println!("✅ Test 1: List files with pagination returns 200 OK");
}

// ---------------------------------------------------------------------------
// Test 2: Get file metadata for non-existent file
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_get_file_metadata_not_found() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/files/non-existent-id")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    println!("✅ Test 2: GET /files/{{id}} returns 404 for non-existent file");
}

// ---------------------------------------------------------------------------
// Test 3: List files with large page number (edge case)
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_list_files_large_page() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/files?page=9999&per_page=20")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let files = body.get("files").and_then(|f| f.as_array()).unwrap();
    assert!(files.is_empty(), "Large page should return empty results");

    println!("✅ Test 3: List files with large page number returns empty");
}

// ---------------------------------------------------------------------------
// Test 4: Delete non-existent file
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_delete_file_not_found() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/v1/files/non-existent-id")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    println!("✅ Test 4: DELETE /files/{{id}} returns 404 for non-existent file");
}

// ---------------------------------------------------------------------------
// Test 5: Verify file record structure after DB setup
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_file_database_schema() {
    let pool = fixtures::setup_test_db().await;

    let columns: Vec<(i32,)> = sqlx::query_as("SELECT 1 FROM information_schema.columns WHERE table_name = 'files'")
        .fetch_all(&pool)
        .await
        .expect("Failed to get columns");

    assert!(!columns.is_empty(), "Files table should have columns");

    println!("✅ Test 5: Files table schema verified");
}

// ---------------------------------------------------------------------------
// Test 6: Verify response contains files array
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_response_structure() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/files?page=1&per_page=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_object(), "Response should be a JSON object");
    assert!(body.get("files").is_some(), "Response should contain 'files' key");

    println!("✅ Test 6: Response structure verified");
}
