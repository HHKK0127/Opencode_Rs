#![cfg(feature = "postgres")]

use actix_web::{test, web, App};
use std::sync::Arc;

async fn setup_test_state() -> AppState {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/opencode_test".to_string()
    });

    let pool = PgPool::connect(&db_url)
        .await
        .expect("PostgreSQL connection required. Set DATABASE_URL env var.");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("users table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            original_name TEXT,
            size BIGINT NOT NULL,
            mime_type TEXT,
            checksum TEXT,
            path TEXT,
            user_id TEXT,
            is_public BOOLEAN DEFAULT FALSE,
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("files table");

    let settings = Settings::default();
    let storage: Arc<dyn opencode_poc::storage::StorageBackend> =
        Arc::new(LocalStorageBackend::new("./test_uploads"));

    AppState::new(settings, pool, storage, None)
}

#[actix_rt::test]
async fn test_health_ready_returns_200() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/ready")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_health_ready_body_has_ready_true() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/ready")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(body["ready"], true);
    assert!(body["components"]["database"]["status"].as_str() == Some("ok"));
    assert!(body["components"]["database"]["latency_ms"].is_number());
}

#[actix_rt::test]
async fn test_health_live_returns_200() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_health_live_body() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(body["alive"], true);
    assert_eq!(body["uptime_check"], "ok");
    assert!(body["timestamp"].as_str().is_some());
}

#[actix_rt::test]
async fn test_health_live_no_auth_required() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(opencode_poc::auth_middleware::AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_health_ready_cache_unavailable_still_ready() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/ready")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(body["ready"], true);
    assert_eq!(body["components"]["cache"]["status"], "unavailable");
}

#[actix_rt::test]
async fn test_request_id_middleware_generates_header() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(opencode_poc::middleware::RequestId)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);
    let request_id = resp.headers().get("x-request-id");
    assert!(
        request_id.is_some(),
        "x-request-id header should be present"
    );

    let rid = request_id.unwrap().to_str().unwrap();
    assert_eq!(rid.len(), 36, "request id should be UUID format");
}

#[actix_rt::test]
async fn test_request_id_middleware_propagates_client_id() {
    let state = setup_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(opencode_poc::middleware::RequestId)
            .configure(api::configure),
    )
    .await;

    let custom_id = "my-trace-id-abc123";
    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .insert_header(("x-request-id", custom_id))
        .to_request();
    let resp = test::call_service(&app, req).await;

    let returned_id = resp
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();

    assert_eq!(
        returned_id, custom_id,
        "client-provided request id should be echoed back"
    );
}
