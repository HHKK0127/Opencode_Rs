use actix_web::{web, App, HttpServer, HttpResponse};
use std::sync::Arc;
use tracing::{info, error, warn};

mod app_state;
mod cache;
mod config;
mod database;
mod error;
mod models;
mod auth_middleware;
mod middleware_cors;
mod middleware_rate_limit;
mod middleware_logging;
mod middleware;
mod api;
mod db;
mod storage;
mod validation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    middleware_logging::init_logging();

    info!("🚀 OpenCode Server Starting...");

    // メトリクス初期化
    middleware::metrics::init_metrics()
        .expect("Failed to initialize metrics");

    cache::register_redis_metrics(&middleware::metrics::REGISTRY)
        .expect("Failed to register Redis metrics");

    // 設定読み込み
    let settings = config::Settings::new()
        .unwrap_or_else(|e| {
            warn!("Config load failed ({}), using defaults", e);
            config::Settings::default()
        });

    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    info!("📋 Environment: {}", env);
    info!("🔧 Server: {}:{}", settings.server.host, settings.server.port);

    // アップロードディレクトリ作成
    let _ = std::fs::create_dir_all(&settings.upload.directory);

    // データベース接続プール作成
    let db_config = database::DatabaseConfig::from_settings(&settings);
    let pool = match database::create_pool(&db_config).await {
        Ok(pool) => {
            info!("✅ Database pool created");
            pool
        }
        Err(e) => {
            error!("❌ Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    // スキーマ初期化
    if let Err(e) = initialize_db(&pool).await {
        error!("Failed to initialize DB schema: {}", e);
        std::process::exit(1);
    }
    info!("✅ Database schema ready");

    // Redisキャッシュ初期化
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let redis_cache = match cache::RedisCache::new(cache::RedisCacheConfig {
        url: redis_url.clone(),
        max_connections: 50,
        connection_timeout_ms: 5000,
    })
    .await
    {
        Ok(cache) => {
            info!("✅ Redis cache connected");
            Some(Arc::new(cache))
        }
        Err(e) if *config::REDIS_REQUIRED => {
            error!("❌ Redis is required but connection failed: {}", e);
            std::process::exit(1);
        }
        Err(e) => {
            warn!("⚠️  Redis unavailable, continuing without cache: {}", e);
            None
        }
    };

    if let Some(ref cache) = redis_cache {
        match cache.health_check().await {
            Ok(_) => info!("Redis health check passed"),
            Err(e) => warn!("Redis health check failed: {}", e),
        }
    }

    // DB最適化
    db::optimize_database(&pool).await.ok();
    db::analyze_tables(&pool).await.ok();

    // ストレージバックエンド初期化
    let storage_backend: Arc<dyn storage::StorageBackend> = Arc::new(
        storage::local_backend::LocalStorageBackend::new(&settings.upload.directory)
    );
    info!("✅ StorageBackend initialized");

    // AppState作成
    let app_state = app_state::AppState::new(
        settings.clone(),
        pool,
        storage_backend,
        redis_cache,
    );
    let bind_addr = app_state.server_addr();

    info!("🔄 Starting server on http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware_cors::configure_cors())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(middleware::RequestId)
            .wrap(middleware::metrics::MetricsMiddleware)
            .wrap(auth_middleware::AuthMiddleware)
            .route("/health", web::get().to(health_check_handler))
            .route("/health/db", web::get().to(db_health_check_handler))
            .route("/metrics", web::get().to(metrics_endpoint))
            .configure(api::configure)
    })
    .workers(settings.server.workers)
    .bind(&bind_addr)?
    .run()
    .await
}

async fn health_check_handler() -> HttpResponse {
    #[derive(serde::Serialize)]
    struct HealthStatus {
        status: String,
        version: String,
        timestamp: String,
    }
    
    HttpResponse::Ok().json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn db_health_check_handler(app_state: web::Data<app_state::AppState>) -> HttpResponse {
    let start = std::time::Instant::now();
    
    match database::health_check(&app_state.db).await {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            HttpResponse::Ok().json(serde_json::json!({
                "status": "database_connected",
                "connected": true,
                "latency_ms": elapsed,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "database_unavailable",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
}

async fn initialize_db(pool: &sqlx::AnyPool) -> Result<(), sqlx::Error> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    // Files table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            original_name TEXT,
            size BIGINT NOT NULL,
            mime_type TEXT,
            checksum TEXT,
            path TEXT,
            user_id TEXT,
            description TEXT,
            tags TEXT,
            is_public BOOLEAN DEFAULT FALSE,
            expires_at TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            s3_path TEXT,
            s3_etag TEXT,
            s3_version_id TEXT,
            storage_type TEXT DEFAULT 'local',
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    // Upload sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS upload_sessions (
            id TEXT PRIMARY KEY,
            file_id TEXT,
            user_id TEXT,
            total_size BIGINT NOT NULL,
            uploaded_size BIGINT DEFAULT 0,
            chunk_size BIGINT DEFAULT 1048576,
            status TEXT CHECK (status IN ('pending', 'uploading', 'completed', 'failed')),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    // 開発環境のみテストユーザー作成
    if std::env::var("ENVIRONMENT").unwrap_or_default() == "development" {
        create_test_user(pool).await.ok();
    }

    Ok(())
}

async fn create_test_user(pool: &sqlx::AnyPool) -> Result<(), sqlx::Error> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand_core::OsRng;

    let test_password = std::env::var("TEST_USER_PASSWORD")
        .unwrap_or_else(|_| {
            let random_pass: String = (0..32)
                .map(|_| {
                    let b = rand::random::<u8>();
                    let idx = b % 62;
                    if idx < 26 {
                        (b'A' + idx) as char
                    } else if idx < 52 {
                        (b'a' + (idx - 26)) as char
                    } else {
                        (b'0' + (idx - 52)) as char
                    }
                })
                .collect();
            warn!("Generated random test password (set TEST_USER_PASSWORD to override)");
            random_pass
        });

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(test_password.as_bytes(), &salt)
        .map_err(|_| sqlx::Error::Configuration("Failed to hash password".into()))?
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash) 
        VALUES ($1, $2, $3) 
        ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash
        "#
    )
    .bind("test-user-1")
    .bind("testuser")
    .bind(&password_hash)
    .execute(pool)
    .await?;

    info!("Test user ready: testuser");
    Ok(())
}

async fn metrics_endpoint() -> HttpResponse {
    match middleware::metrics::get_metrics() {
        Ok(metrics) => HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to collect metrics"),
    }
}
