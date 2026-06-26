// Users API Integration Tests (PostgreSQL)

use actix_web::{http::StatusCode, test, web, App};
use opencode_poc::api;

mod fixtures;

// ---------------------------------------------------------------------------
// Test 1: List users returns an array
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_list_users_success() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array of users");
}

// ---------------------------------------------------------------------------
// Test 2: Get user by ID returns 404 for non-existent user
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_get_user_not_found() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users/non-existent-id")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Test 3: User endpoints are under /api/v1 scope
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_users_routing() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
