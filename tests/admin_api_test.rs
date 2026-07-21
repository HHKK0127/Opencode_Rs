#![cfg(feature = "postgres")]

// Admin API Integration Tests (PostgreSQL)

use actix_web::{http::StatusCode, test, web, App};
use opencode_poc::api;

mod fixtures;

// ---------------------------------------------------------------------------
// Test 1: Database status endpoint returns stats
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_admin_db_status() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/db/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "DB status should return success"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
}

// ---------------------------------------------------------------------------
// Test 2: Migration history endpoint returns list
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_admin_migration_history() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/db/migrations")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Migration history should return success"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Database analyze endpoint succeeds
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_admin_analyze_database() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/db/analyze")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Analyze database should return success"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Admin endpoints return proper JSON structure
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_admin_db_status_structure() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/db/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(
        body.get("database").is_some(),
        "Response should contain database info"
    );
}
