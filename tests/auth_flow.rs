use actix_web::{test, web, App};
use opencode_poc::api;
use opencode_poc::app_state::AppState;

mod fixtures;

#[actix_rt::test]
async fn test_register_new_user_success() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let unique = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": format!("newuser_{}", unique),
            "password": "SecurePassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_rt::test]
async fn test_login_with_valid_credentials() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, password) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_login_with_invalid_password() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, _) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&serde_json::json!({
            "username": username,
            "password": "WrongPassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_token_refresh() {
    let app_state = fixtures::create_test_app_state().await;
    let old_token = fixtures::create_test_token("test-user-id", &app_state.settings.auth.jwt_secret);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(&serde_json::json!({
            "token": old_token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_password_validation_min_length() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": "newuser",
            "password": "short"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_invalid_token_refresh() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(api::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(&serde_json::json!({
            "token": "invalid.token.format"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
