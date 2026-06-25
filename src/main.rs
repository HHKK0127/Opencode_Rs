use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand_core::OsRng;

mod app_state;
mod cache;  // Wave 4: Redis caching layer
mod config;
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

    // Initialize Prometheus metrics
    middleware::metrics::init_metrics()
        .expect("Failed to initialize metrics");

    // Initialize Redis cache metrics
    cache::register_redis_metrics(&middleware::metrics::REGISTRY)
        .expect("Failed to register Redis metrics");

    println!("🚀 PoC Verification Server Starting...");

    let settings = config::Settings::new()
        .unwrap_or_else(|_| config::Settings::default());

    println!("📋 Environment: {}", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()));
    println!("🔧 Server: {}:{}", settings.server.host, settings.server.port);

    // Ensure uploads directory exists
    let _ = std::fs::create_dir_all(&settings.upload.directory);

    // Resolve DATABASE_URL: env var takes precedence over config file
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| settings.database.url.clone());

    // Connect to PostgreSQL
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL. Set DATABASE_URL env var.");

    println!("✅ Database connected: {}", database_url.split('@').last().unwrap_or("(hidden)"));

    // Initialize schema
    initialize_db(&pool).await.expect("Failed to initialize DB schema");
    println!("✅ Database schema ready");

    // Initialize Redis cache (Wave 4)
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:test_password@127.0.0.1:6379".to_string());
    let redis_cache = cache::RedisCache::new(cache::RedisCacheConfig {
        url: redis_url.clone(),
        max_connections: 50,
        connection_timeout_ms: 5000,
    })
    .await
    .map_err(|e| {
        eprintln!("⚠️  Redis connection failed: {}. Continuing without cache...", e);
        e
    });

    if let Ok(ref cache) = redis_cache {
        match cache.health_check().await {
            Ok(_) => println!("✅ Redis cache connected: {}", redis_url),
            Err(e) => eprintln!("⚠️  Redis health check failed: {}", e),
        }
    }

    // Apply database optimizations
    db::optimize_database(&pool)
        .await
        .expect("Failed to apply database optimizations");

    db::analyze_tables(&pool)
        .await
        .expect("Failed to analyze database tables");

    // Display database stats
    match db::get_database_stats(&pool).await {
        Ok(stats) => {
            println!("📊 Database: {:.2} MB (journal: {})",
                stats.total_size_bytes as f64 / (1024.0 * 1024.0),
                stats.journal_mode);
        }
        Err(e) => eprintln!("⚠️  Failed to get database stats: {}", e),
    }

    println!("🔄 Starting server on http://{}:{}", settings.server.host, settings.server.port);

    // Initialize StorageBackend (Local for dev, S3 for prod)
    let storage_backend: Arc<dyn storage::StorageBackend> = Arc::new(
        storage::local_backend::LocalStorageBackend::new(&settings.upload.directory)
    );
    println!("✅ StorageBackend initialized (Local)");

    // Create AppState
    let app_state = app_state::AppState::new(
        settings.clone(),
        pool,
        storage_backend,
        redis_cache.ok().map(Arc::new),
    );
    let bind_addr = app_state.server_addr();

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
    use serde::Serialize;
    #[derive(Serialize)]
    struct HealthStatus { status: String, version: String, timestamp: String }
    HttpResponse::Ok().json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn db_health_check_handler(app_state: web::Data<app_state::AppState>) -> HttpResponse {
    match sqlx::query("SELECT 1").fetch_one(&app_state.db).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "database_connected",
            "connected": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        Err(e) => {
            eprintln!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "database_unavailable",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
}

async fn initialize_db(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Files table (full schema — avoids ALTER TABLE drift)
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
        "#,
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
        "#,
    )
    .execute(pool)
    .await?;

    // Create test user (idempotent)
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(b"testpassword", &salt)
        .map_err(|_| sqlx::Error::Configuration("Failed to hash password".into()))?
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
    )
    .bind("test-user-1")
    .bind("testuser")
    .bind(&password_hash)
    .execute(pool)
    .await?;

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
