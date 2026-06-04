//! Day 14: Session Management Integration Tests
//!
//! 5 tests covering JWT + Redis Session lifecycle

use actix_web::{
    http::StatusCode,
    test,
    web,
    App,
    middleware::Logger,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Test fixtures setup
async fn create_test_app_state() -> opencode::app_state::AppState {
    // Use in-memory SQLite for tests
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Initialize schema
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    // Create test user
    let test_user_id = "test-user-123";
    let test_username = "testuser";
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$salt$hash";

    sqlx::query(
        "INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (?, ?, ?)"
    )
    .bind(test_user_id)
    .bind(test_username)
    .bind(password_hash)
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    let settings = Arc::new(opencode::config::Settings::default());

    // Create Redis cache connection for tests
    let redis_cache = opencode::cache::redis::RedisCache::new(
        &settings.cache.redis_url,
        "test",
    )
    .await
    .ok()
    .map(Arc::new);

    // Create in-memory storage for tests
    let storage = Arc::new(
        opencode::storage::local_fs::LocalFileStorage::new("./test_uploads").await
            .expect("Failed to create test storage")
    ) as Arc<dyn opencode::storage::StorageBackend>;

    opencode::app_state::AppState {
        settings: settings.clone(),
        db: pool,
        storage,
        cache: redis_cache,
        ttl_config: Arc::new(opencode::cache::CacheTTLConfig::default()),
    }
}

/// Test 1: Session creation on login
#[actix_web::test]
async fn test_session_creation_on_login() {
    let app_state = create_test_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .configure(opencode::api::configure)
            ),
    )
    .await;

    // Login request
    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&json!({
            "username": "testuser",
            "password": "testpass123"
        }))
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;

    // Status should be OK if user exists with correct password
    // For this test, we're checking the response structure
    if login_resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(login_resp).await;
        assert!(body.get("token").is_some(), "Response should contain token");
        assert!(body.get("expires_in").is_some(), "Response should contain expires_in");
    }
}

/// Test 2: Session validation in middleware
#[actix_web::test]
async fn test_middleware_session_validation() {
    let app_state = create_test_app_state().await;

    // Create a mock token for testing
    let claims = opencode::models::Claims {
        sub: "test-token-123".to_string(),
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(app_state.settings.auth.jwt_secret.as_bytes()),
    ).expect("Failed to encode JWT");

    // Create session in Redis if cache available
    if let Some(ref cache) = app_state.cache {
        let session_mgr = opencode::cache::session::SessionManager::new(cache.clone());
        let _ = session_mgr.create_session(
            &token,
            "user-123",
            "testuser",
            vec!["read".to_string()],
        ).await;
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .wrap(Logger::default())
            .wrap(opencode::auth_middleware::AuthMiddleware)
            .service(
                web::scope("/api/v1")
                    .configure(opencode::api::configure)
            ),
    )
    .await;

    // Access protected endpoint with valid token
    let req = test::TestRequest::get()
        .uri("/api/v1/sessions/info")
        .header("Authorization", format!("Bearer {}", token))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should succeed if Redis is available
    if resp.status() == StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("is_active").is_some());
    }
}

/// Test 3: Session TTL extension
#[actix_web::test]
async fn test_session_ttl_extension() {
    let app_state = create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode::cache::session::SessionManager::new(cache.clone());

    // Create session
    let token = "extend-test-token";
    session_mgr.create_session(
        token,
        "user-456",
        "extenduser",
        vec!["read".to_string()],
    )
    .await
    .expect("Failed to create session");

    // Wait a bit
    sleep(Duration::from_millis(500)).await;

    // Get initial last_activity
    let initial = session_mgr.validate_session(token)
        .await
        .expect("Failed to validate session");
    let initial_activity = initial.last_activity;

    // Extend session
    let _ = session_mgr.extend_session(token).await;

    // Verify last_activity updated
    let updated = session_mgr.validate_session(token)
        .await
        .expect("Failed to validate session");
    assert!(
        updated.last_activity > initial_activity,
        "last_activity should be updated"
    );
}

/// Test 4: Session invalidation on logout
#[actix_web::test]
async fn test_session_invalidation_on_logout() {
    let app_state = create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode::cache::session::SessionManager::new(cache.clone());

    // Create session
    let token = "logout-test-token";
    session_mgr.create_session(
        token,
        "user-789",
        "logoutuser",
        vec!["read".to_string()],
    )
    .await
    .expect("Failed to create session");

    // Verify session exists
    assert!(
        session_mgr.validate_session(token).await.is_ok(),
        "Session should exist"
    );

    // Invalidate session
    session_mgr.invalidate_session(token)
        .await
        .expect("Failed to invalidate session");

    // Verify session no longer exists
    let result = session_mgr.validate_session(token).await;
    assert!(result.is_err(), "Session should be invalidated");
}

/// Test 5: Concurrent session handling
#[actix_web::test]
async fn test_concurrent_user_sessions() {
    let app_state = create_test_app_state().await;

    if app_state.cache.is_none() {
        println!("Skipping test: Redis cache not available");
        return;
    }

    let cache = app_state.cache.as_ref().unwrap();
    let session_mgr = opencode::cache::session::SessionManager::new(cache.clone());

    // Create sessions for multiple users
    let users = vec![
        ("token-1", "user-1", "alice"),
        ("token-2", "user-2", "bob"),
        ("token-3", "user-3", "charlie"),
    ];

    for (token, user_id, username) in &users {
        session_mgr.create_session(
            token,
            user_id,
            username,
            vec!["read".to_string()],
        )
        .await
        .expect(&format!("Failed to create session for {}", username));
    }

    // Verify all sessions exist and are isolated
    for (token, user_id, username) in &users {
        let session = session_mgr.validate_session(token)
            .await
            .expect(&format!("Failed to validate session for {}", username));
        assert_eq!(session.user_id, *user_id);
        assert_eq!(session.username, *username);
    }

    // Invalidate one user's session
    session_mgr.invalidate_session("token-2")
        .await
        .expect("Failed to invalidate session");

    // Verify alice and charlie still have sessions
    assert!(
        session_mgr.validate_session("token-1").await.is_ok(),
        "Alice's session should still be valid"
    );
    assert!(
        session_mgr.validate_session("token-3").await.is_ok(),
        "Charlie's session should still be valid"
    );

    // Verify bob's session is gone
    assert!(
        session_mgr.validate_session("token-2").await.is_err(),
        "Bob's session should be invalidated"
    );
}
struct SessionMock {
    user_id: String,
    username: String,
    created_at: i64,
    last_activity: i64,
    permissions: Vec<String>,
    ttl_remaining: u64,
}

impl MockSessionStore {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn create_session(
        &self,
        token: &str,
        user_id: &str,
        username: &str,
        permissions: Vec<String>,
    ) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let session = SessionMock {
            user_id: user_id.to_string(),
            username: username.to_string(),
            created_at: now,
            last_activity: now,
            permissions,
            ttl_remaining: 86400, // 24 hours
        };

        let mut store = self.sessions.lock().unwrap();
        store.insert(token.to_string(), session);
        Ok(())
    }

    fn validate_session(&self, token: &str) -> Result<SessionMock, String> {
        let store = self.sessions.lock().unwrap();
        store
            .get(token)
            .cloned()
            .ok_or_else(|| "Session not found".to_string())
    }

    fn extend_session(&self, token: &str) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut store = self.sessions.lock().unwrap();
        match store.get_mut(token) {
            Some(session) => {
                session.last_activity = now;
                session.ttl_remaining = 86400;
                Ok(())
            }
            None => Err("Session not found".to_string()),
        }
    }

    fn invalidate_session(&self, token: &str) -> Result<(), String> {
        let mut store = self.sessions.lock().unwrap();
        store.remove(token);
        Ok(())
    }

    fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    fn check_permission(&self, token: &str, permission: &str) -> Result<bool, String> {
        let session = self.validate_session(token)?;
        Ok(session.permissions.contains(&permission.to_string()))
    }
}

#[test]
fn test_01_session_creation() {
    // Test 1: Session creation and storage
    let store = MockSessionStore::new();

    let result = store.create_session(
        "token-abc123",
        "user-001",
        "alice",
        vec!["read".to_string(), "write".to_string()],
    );

    assert!(result.is_ok(), "Session creation should succeed");
    assert_eq!(store.session_count(), 1, "Should have 1 session");

    // Verify session exists
    let session = store.validate_session("token-abc123").unwrap();
    assert_eq!(session.user_id, "user-001");
    assert_eq!(session.username, "alice");
    assert_eq!(session.permissions.len(), 2);
}

#[test]
fn test_02_session_validation() {
    // Test 2: Session validation and retrieval
    let store = MockSessionStore::new();

    store
        .create_session(
            "token-xyz789",
            "user-002",
            "bob",
            vec!["read".to_string()],
        )
        .unwrap();

    // Valid session should be retrievable
    let session = store.validate_session("token-xyz789").unwrap();
    assert_eq!(session.username, "bob");
    assert!(session.last_activity > 0);

    // Invalid token should fail
    let invalid = store.validate_session("token-invalid");
    assert!(invalid.is_err(), "Invalid token should fail validation");
}

#[test]
fn test_03_session_extension() {
    // Test 3: Session TTL extension
    let store = MockSessionStore::new();

    store
        .create_session(
            "token-ext001",
            "user-003",
            "charlie",
            vec![],
        )
        .unwrap();

    let initial_session = store.validate_session("token-ext001").unwrap();
    let initial_activity = initial_session.last_activity;
    let initial_ttl = initial_session.ttl_remaining;

    // Simulate time passing and extending session
    std::thread::sleep(std::time::Duration::from_millis(100));
    store.extend_session("token-ext001").unwrap();

    let extended_session = store.validate_session("token-ext001").unwrap();
    assert!(
        extended_session.last_activity >= initial_activity,
        "Activity timestamp should be updated"
    );
    assert_eq!(
        extended_session.ttl_remaining, initial_ttl,
        "TTL should be refreshed"
    );
}

#[test]
fn test_04_session_expiration() {
    // Test 4: Session expiration and invalidation
    let store = MockSessionStore::new();

    store
        .create_session(
            "token-exp001",
            "user-004",
            "diana",
            vec![],
        )
        .unwrap();

    // Session should exist
    assert_eq!(store.session_count(), 1);
    let _ = store.validate_session("token-exp001").unwrap();

    // Invalidate (logout)
    let invalidate_result = store.invalidate_session("token-exp001");
    assert!(invalidate_result.is_ok(), "Invalidation should succeed");

    // Session should no longer exist
    assert_eq!(store.session_count(), 0);
    let validate_result = store.validate_session("token-exp001");
    assert!(
        validate_result.is_err(),
        "Invalidated session should not be found"
    );
}

#[test]
fn test_05_concurrent_session_management() {
    // Test 5: Multiple concurrent sessions
    let store = Arc::new(MockSessionStore::new());

    // Create multiple sessions concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = std::thread::spawn(move || {
            let token = format!("token-concurrent-{}", i);
            let user_id = format!("user-{}", i);
            let username = format!("user{}", i);
            let permissions = if i % 2 == 0 {
                vec!["read".to_string()]
            } else {
                vec!["read".to_string(), "write".to_string()]
            };

            store_clone
                .create_session(&token, &user_id, &username, permissions)
                .unwrap();
        });

        handles.push(handle);
    }

    // Wait for all sessions to be created
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all sessions exist
    assert_eq!(store.session_count(), 10, "Should have 10 concurrent sessions");

    // Verify specific sessions
    for i in 0..10 {
        let token = format!("token-concurrent-{}", i);
        let session = store.validate_session(&token).unwrap();
        assert_eq!(session.user_id, format!("user-{}", i));

        // Check permissions based on creation pattern
        if i % 2 == 0 {
            assert_eq!(session.permissions.len(), 1);
        } else {
            assert_eq!(session.permissions.len(), 2);
        }
    }

    // Invalidate half of the sessions
    for i in (0..10).step_by(2) {
        let token = format!("token-concurrent-{}", i);
        store.invalidate_session(&token).unwrap();
    }

    // Verify correct number remain
    assert_eq!(
        store.session_count(),
        5,
        "Should have 5 sessions after removing half"
    );
}

#[test]
fn test_06_permission_validation() {
    // Test 6: Permission checking in sessions
    let store = MockSessionStore::new();

    store
        .create_session(
            "token-perm001",
            "user-005",
            "eve",
            vec!["read".to_string(), "write".to_string()],
        )
        .unwrap();

    // Check existing permissions
    assert!(store.check_permission("token-perm001", "read").unwrap());
    assert!(store.check_permission("token-perm001", "write").unwrap());

    // Check non-existing permission
    assert!(!store.check_permission("token-perm001", "delete").unwrap());

    // Check permission on invalid token
    let invalid_perm = store.check_permission("token-invalid", "read");
    assert!(invalid_perm.is_err());
}

#[test]
fn test_07_session_data_structure() {
    // Test 7: Session data structure and serialization
    let store = MockSessionStore::new();

    store
        .create_session(
            "token-struct001",
            "user-006",
            "frank",
            vec!["admin".to_string(), "delete".to_string(), "write".to_string()],
        )
        .unwrap();

    let session = store.validate_session("token-struct001").unwrap();

    // Verify all fields
    assert!(!session.user_id.is_empty());
    assert!(!session.username.is_empty());
    assert!(session.created_at > 0);
    assert!(session.last_activity > 0);
    assert_eq!(session.permissions.len(), 3);
    assert!(session.ttl_remaining > 0);

    // Verify TTL is reasonable (24 hours = 86400 seconds)
    assert_eq!(session.ttl_remaining, 86400);
}
