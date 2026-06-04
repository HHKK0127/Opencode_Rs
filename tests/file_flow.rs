use actix_web::{test, web, App};
use opencode_poc::api;

mod fixtures;

#[actix_rt::test]
async fn test_health_endpoint_for_file_tests() {
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
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_api_responds_to_requests() {
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
async fn test_cors_headers_present() {
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
    assert!(resp.headers().contains_key("access-control-allow-origin"));
}

#[actix_rt::test]
async fn test_invalid_route_returns_404() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/nonexistent")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn test_json_content_type_validation() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_payload("not json")
        .insert_header(("Content-Type", "text/plain"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_multiple_concurrent_requests() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    for _ in 0..3 {
        let req = test::TestRequest::get()
            .uri("/api/v1/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
