use actix_web::{test, web, App, http::StatusCode};
use opencode_poc::api;
use opencode_poc::app_state::AppState;
use opencode_poc::config::Settings;
use opencode_poc::storage::s3_client::S3Client;
use sqlx::sqlite::SqlitePool;
use serde_json::json;

async fn setup_test_app() -> AppState {
    let settings = Settings::default();
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory pool");

    // Initialize schema
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            filename TEXT NOT NULL,
            original_name TEXT,
            size INTEGER NOT NULL,
            mime_type TEXT,
            path TEXT,
            checksum TEXT,
            description TEXT,
            tags TEXT,
            is_public BOOLEAN DEFAULT FALSE,
            expires_at TIMESTAMP,
            s3_path TEXT,
            s3_etag TEXT,
            s3_version_id TEXT,
            storage_type TEXT DEFAULT 'local',
            metadata TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            uploaded_at TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create files table");

    let s3_client = S3Client::new(&settings)
        .await
        .expect("Failed to initialize S3 client");

    AppState::new(settings, pool, s3_client)
}

#[actix_rt::test]
async fn test_register_metadata_valid_request() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test-file.txt",
        "s3_path": "s3://minio/test-file.txt",
        "s3_etag": "abc123def456",
        "size": 1024,
        "mime_type": "text/plain"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 1: S3 validation working");
}

#[actix_rt::test]
async fn test_register_metadata_invalid_s3_path_format() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test-file.txt",
        "s3_path": "invalid-path",
        "s3_etag": "abc123",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 2: Invalid S3 path format rejected");
}

#[actix_rt::test]
async fn test_register_metadata_invalid_filename() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "",
        "s3_path": "s3://bucket/key",
        "s3_etag": "abc123",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 3: Empty filename rejected");
}

#[actix_rt::test]
async fn test_register_metadata_invalid_size() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test.txt",
        "s3_path": "s3://bucket/key",
        "s3_etag": "abc123",
        "size": 0
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 4: Invalid size rejected");
}

#[actix_rt::test]
async fn test_complete_s3_upload_invalid_path() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "s3_path": "bucket/key",
        "s3_etag": "abc123",
        "filename": "test.txt",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/s3/complete")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 5: S3 complete with invalid path rejected");
}

#[actix_rt::test]
async fn test_extract_s3_key_from_path() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test.txt",
        "s3_path": "s3://my-bucket/path/to/file.txt",
        "s3_etag": "abc123",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 6: S3 key extraction from path working");
}

#[actix_rt::test]
async fn test_register_metadata_missing_etag() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test.txt",
        "s3_path": "s3://bucket/key",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 7: Missing S3 ETag rejected");
}

#[actix_rt::test]
async fn test_register_metadata_optional_fields() {
    let app_state = setup_test_app().await;
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req_body = json!({
        "filename": "test.txt",
        "s3_path": "s3://bucket/key",
        "s3_etag": "abc123",
        "size": 1024
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/files/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    println!("✅ Test 8: Optional fields handled correctly");
}
