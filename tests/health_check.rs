use actix_web::{test, web, App};
use opencode_poc::api;
use opencode_poc::app_state::AppState;

mod fixtures;

#[actix_rt::test]
async fn test_health_endpoint() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_health_db_endpoint() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/db")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_cors_headers_on_health() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(opencode_poc::middleware_cors::configure_cors())
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .insert_header(("Origin", "http://localhost:3000"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_health_endpoint_no_auth_required() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(opencode_poc::auth_middleware::AuthMiddleware)
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
