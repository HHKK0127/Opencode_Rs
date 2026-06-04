use actix_web::{test, web, App};
use opencode_poc::api;
use opencode_poc::app_state::AppState;

mod fixtures {
    pub use tests::fixtures::*;
}

#[actix_rt::test]
async fn test_health_endpoint() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body.get("status").unwrap(), "healthy");
    assert!(body.get("version").is_some());
    assert!(body.get("timestamp").is_some());
}

#[actix_rt::test]
async fn test_health_db_endpoint() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::get()
        .uri("/health/db")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body.get("status").unwrap(), "database_connected");
}

#[actix_rt::test]
async fn test_cors_headers_on_health() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(opencode_poc::middleware_cors::configure_cors())
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Origin", "http://localhost:3000"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert!(resp.headers().contains_key("access-control-allow-origin"));
}

#[actix_rt::test]
async fn test_health_endpoint_no_auth_required() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(opencode_poc::auth_middleware::AuthMiddleware)
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    // No Authorization header
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
