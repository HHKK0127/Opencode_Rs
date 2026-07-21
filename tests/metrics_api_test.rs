#![cfg(feature = "postgres")]

// Metrics API Integration Tests

use actix_web::{http::StatusCode, test, web, App};
use opencode_poc::api;
use opencode_poc::middleware::metrics::{init_metrics, MetricsMiddleware};

mod fixtures;

fn ensure_metrics_init() {
    let _ = init_metrics();
}

// ---------------------------------------------------------------------------
// Test 1: Metrics endpoint returns Prometheus format
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_metrics_endpoint_available() {
    ensure_metrics_init();
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .wrap(MetricsMiddleware)
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    // Make a request first to populate metrics
    let health_req = test::TestRequest::get().uri("/health").to_request();
    let _ = test::call_service(&app, health_req).await;

    let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_text = String::from_utf8_lossy(&body);

    assert!(
        body_text.contains("http_requests_total")
            || body_text.contains("http_request_duration_seconds"),
        "Metrics should contain HTTP metrics"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Metrics endpoint returns plain text
// ---------------------------------------------------------------------------
#[actix_rt::test]
async fn test_metrics_content_type() {
    ensure_metrics_init();
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .wrap(MetricsMiddleware)
            .app_data(web::Data::new(app_state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();

    let resp = test::call_service(&app, req).await;
    let content_type = resp.headers().get("content-type");

    assert!(content_type.is_some());
    assert!(
        content_type
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/plain"),
        "Metrics should be text/plain"
    );
}
