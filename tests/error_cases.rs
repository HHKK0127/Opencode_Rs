use actix_web::{test, web, App};
use opencode_poc::api;
use opencode_poc::app_state::AppState;

mod fixtures;

#[actix_rt::test]
async fn test_invalid_json_payload() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_payload("not valid json")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_register_duplicate_username() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, _) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": username,
            "password": "ValidPassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

#[actix_rt::test]
async fn test_missing_required_auth_header() {
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

    let req = test::TestRequest::post()
        .uri("/api/v1/users/profile")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_malformed_auth_header() {
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

    let req = test::TestRequest::post()
        .uri("/api/v1/users/profile")
        .insert_header(("Authorization", "InvalidAuthFormat"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
