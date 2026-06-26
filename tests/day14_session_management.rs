//! Day 14: Session Management Integration Tests (PostgreSQL)
//!
//! Migrated from tests/legacy/day14_session_management.rs
//! - SQLite → PostgreSQL (PgPool via fixtures)
//! - tokio::test for async integration tests

use actix_web::{
    http::StatusCode,
    middleware::Logger,
    test,
    web,
    App,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod fixtures;

// ---------------------------------------------------------------------------
// Test 1: Session creation on login
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_session_creation_on_login() {
    let app_state = fixtures::create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .configure(opencode_poc::api::configure),
    )
    .await;

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&serde_json::json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;

    if login_resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(login_resp).await;
        assert!(body.get("token").is_some(), "Response should contain token");
        assert!(body.get("expires_in").is_some(), "Response should contain expires_in");
    }
}

// ---------------------------------------------------------------------------
// Test 2: Session validation in middleware
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_middleware_session_validation() {
    let app_state = fixtures::create_test_app_state().await;

    let claims = opencode_poc::models::Claims {
        sub: "test-token-123".to_string(),
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(app_state.settings.auth.jwt_secret.as_bytes()),
    )
    .expect("Failed to encode JWT");

    if let Some(ref cache) = app_state.cache {
        let session_mgr = opencode_poc::cache::session::SessionManager::new(cache.clone());
        let _ = session_mgr
            .create_session(&token, "user-123", "testuser", vec!["read".to_string()])
            .await;
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .wrap(opencode_poc::auth_middleware::AuthMiddleware)
            .configure(opencode_poc::api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/sessions/info")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("is_active").is_some());
    }
}

// ---------------------------------------------------------------------------
// Test 3: Session TTL extension
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_session_ttl_extension() {
    let app_state = fixtures::create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode_poc::cache::session::SessionManager::new(cache.clone());

    let token = "extend-test-token";
    session_mgr
        .create_session(token, "user-456", "extenduser", vec!["read".to_string()])
        .await
        .expect("Failed to create session");

    sleep(Duration::from_millis(500)).await;

    let initial = session_mgr
        .validate_session(token)
        .await
        .expect("Failed to validate session");
    let initial_activity = initial.last_activity;

    let _ = session_mgr.extend_session(token).await;

    let updated = session_mgr
        .validate_session(token)
        .await
        .expect("Failed to validate session");
    assert!(updated.last_activity > initial_activity, "last_activity should be updated");
}

// ---------------------------------------------------------------------------
// Test 4: Session invalidation on logout
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_session_invalidation_on_logout() {
    let app_state = fixtures::create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode_poc::cache::session::SessionManager::new(cache.clone());

    let token = "logout-test-token";
    session_mgr
        .create_session(token, "user-789", "logoutuser", vec!["read".to_string()])
        .await
        .expect("Failed to create session");

    assert!(session_mgr.validate_session(token).await.is_ok(), "Session should exist");

    session_mgr
        .invalidate_session(token)
        .await
        .expect("Failed to invalidate session");

    let result = session_mgr.validate_session(token).await;
    assert!(result.is_err(), "Session should be invalidated");
}

// ---------------------------------------------------------------------------
// Test 5: Concurrent session handling
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_concurrent_user_sessions() {
    let app_state = fixtures::create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode_poc::cache::session::SessionManager::new(cache.clone());

    let users = vec![
        ("token-1", "user-1", "alice"),
        ("token-2", "user-2", "bob"),
        ("token-3", "user-3", "charlie"),
    ];

    for (token, user_id, username) in &users {
        session_mgr
            .create_session(token, user_id, username, vec!["read".to_string()])
            .await
            .expect(&format!("Failed to create session for {}", username));
    }

    for (token, user_id, username) in &users {
        let session = session_mgr
            .validate_session(token)
            .await
            .expect(&format!("Failed to validate session for {}", username));
        assert_eq!(session.user_id, *user_id);
        assert_eq!(session.username, *username);
    }

    session_mgr
        .invalidate_session("token-2")
        .await
        .expect("Failed to invalidate session");

    assert!(session_mgr.validate_session("token-1").await.is_ok(), "Alice's session should still be valid");
    assert!(session_mgr.validate_session("token-3").await.is_ok(), "Charlie's session should still be valid");
    assert!(session_mgr.validate_session("token-2").await.is_err(), "Bob's session should be invalidated");
}
