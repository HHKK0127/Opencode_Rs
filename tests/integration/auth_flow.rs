use actix_web::{test, web, App};
use opencode_poc::api;
use opencode_poc::app_state::AppState;
use opencode_poc::config::Settings;

mod fixtures {
    pub use tests::fixtures::*;
}

#[actix_rt::test]
async fn test_register_new_user_success() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": "newuser",
            "password": "SecurePassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("id").is_some());
    assert_eq!(body.get("username").unwrap(), "newuser");
}

#[actix_rt::test]
async fn test_register_duplicate_username() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, _) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": username,
            "password": "AnotherPassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_login_with_valid_credentials() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, password) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
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

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("token").is_some());
    assert!(body.get("expires_in").is_some());
}

#[actix_rt::test]
async fn test_login_with_invalid_password() {
    let app_state = fixtures::create_test_app_state().await;
    let (username, _) = fixtures::create_test_user(&app_state.db).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
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
async fn test_login_nonexistent_user() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&serde_json::json!({
            "username": "nonexistent",
            "password": "SomePassword123"
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
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(&serde_json::json!({
            "token": old_token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("token").is_some());
    let new_token = body.get("token").unwrap().as_str().unwrap();
    assert_ne!(new_token, "");
}

#[actix_rt::test]
async fn test_password_validation_min_length() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "username": "newuser",
            "password": "short"  // less than 8 chars
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
            .service(
                web::scope("/api/v1")
                    .configure(api::configure)
            )
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
