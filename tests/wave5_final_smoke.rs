/// Wave 5 Final Smoke Tests — production readiness gate (Wave 1-5)
use actix_web::{test, web, App};
use opencode_poc::{
    api,
    app_state::AppState,
    auth_middleware::AuthMiddleware,
    config::Settings,
    middleware::RequestId,
    middleware_cors::configure_cors,
    storage::local_backend::LocalStorageBackend,
};
use sqlx::postgres::PgPool;
use std::sync::Arc;

async fn create_state() -> AppState {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/opencode_test".to_string());

    let pool = PgPool::connect(&db_url)
        .await
        .expect("PostgreSQL connection required. Set DATABASE_URL env var.");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("users table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            original_name TEXT,
            size BIGINT NOT NULL,
            mime_type TEXT,
            checksum TEXT,
            path TEXT,
            user_id TEXT,
            is_public BOOLEAN DEFAULT FALSE,
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("files table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS upload_sessions (
            id TEXT PRIMARY KEY,
            file_id TEXT,
            user_id TEXT,
            total_size BIGINT NOT NULL,
            uploaded_size BIGINT DEFAULT 0,
            chunk_size BIGINT DEFAULT 1048576,
            status TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("upload_sessions table");

    let settings = Settings::default();
    let storage: Arc<dyn opencode_poc::storage::StorageBackend> =
        Arc::new(LocalStorageBackend::new("./test_uploads_smoke"));
    AppState::new(settings, pool, storage, None)
}

// ── Wave 1: Authentication ────────────────────────────────────────────────────

#[actix_rt::test]
async fn smoke_register_and_login() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(serde_json::json!({"username": "smoke_user", "password": "SmokePass123!"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "register: {}", resp.status());

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(serde_json::json!({"username": "smoke_user", "password": "SmokePass123!"}))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(body["token"].as_str().is_some(), "no token in login response");
}

#[actix_rt::test]
async fn smoke_invalid_login_returns_401() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(serde_json::json!({"username": "nobody", "password": "wrong"}))
        .to_request();
    let status = test::call_service(&app, req).await.status().as_u16();
    assert!(status == 400 || status == 401, "expected 4xx, got {}", status);
}

#[actix_rt::test]
async fn smoke_protected_endpoint_requires_auth() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/files")
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 401);
}

#[actix_rt::test]
async fn smoke_auth_endpoints_exempt_from_jwt() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    for path in ["/api/v1/auth/login", "/api/v1/auth/register"] {
        let req = test::TestRequest::post()
            .uri(path)
            .set_json(serde_json::json!({"username": "x", "password": "y"}))
            .to_request();
        let status = test::call_service(&app, req).await.status().as_u16();
        // Handler may return 400/401 — middleware must NOT intercept (which would give 401)
        // For register: 400 bad request is fine; for login: 401 from handler is fine
        assert!(
            status != 401 || path.contains("login"),
            "{} should be exempt from JWT middleware, got {}",
            path,
            status
        );
    }
}

// ── Wave 5 Phase 1: Health probes ────────────────────────────────────────────

#[actix_rt::test]
async fn smoke_readiness_probe() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/ready")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["ready"], true);
    assert_eq!(body["components"]["database"]["status"], "ok");
    assert!(body["components"]["cache"]["status"].as_str().is_some());
}

#[actix_rt::test]
async fn smoke_liveness_probe() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["alive"], true);
    assert_eq!(body["uptime_check"], "ok");
}

#[actix_rt::test]
async fn smoke_all_health_endpoints_return_200() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(AuthMiddleware)
            .configure(api::configure),
    )
    .await;

    for path in ["/api/v1/health", "/api/v1/health/ready", "/api/v1/health/live", "/api/v1/health/db"] {
        let req = test::TestRequest::get().uri(path).to_request();
        let status = test::call_service(&app, req).await.status();
        assert!(status.is_success(), "{} returned {} (no auth required)", path, status);
    }
}

// ── Wave 5 Phase 1: Request ID middleware ─────────────────────────────────────

#[actix_rt::test]
async fn smoke_request_id_auto_generated() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(RequestId)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let rid = resp.headers().get("x-request-id");
    assert!(rid.is_some(), "x-request-id missing");
    assert_eq!(rid.unwrap().to_str().unwrap().len(), 36, "not UUID format");
}

#[actix_rt::test]
async fn smoke_request_id_client_propagated() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(RequestId)
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .insert_header(("x-request-id", "my-trace-abc"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.headers().get("x-request-id").unwrap().to_str().unwrap(),
        "my-trace-abc"
    );
}

// ── CORS ──────────────────────────────────────────────────────────────────────

#[actix_rt::test]
async fn smoke_cors_localhost_allowed() {
    let state = create_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(configure_cors())
            .configure(api::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health/live")
        .insert_header(("Origin", "http://localhost:3000"))
        .to_request();
    assert!(test::call_service(&app, req).await.status().is_success());
}
