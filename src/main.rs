use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::sqlite::SqlitePool;
use std::path::Path;
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

    println!("🚀 PoC Verification Server Starting...");

    let settings = config::Settings::new()
        .unwrap_or_else(|_| config::Settings::default());

    println!("📋 Environment: {}", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()));
    println!("🔧 Server: {}:{}", settings.server.host, settings.server.port);

    // Ensure uploads directory exists
    let _ = std::fs::create_dir_all(&settings.upload.directory);

    let db_path = &settings.database.path;

    // Initialize database on first run
    let needs_init = !Path::new(db_path).exists();
    if needs_init && settings.database.auto_init {
        std::fs::File::create(db_path).expect("Failed to create DB file");
        initialize_db(db_path).await.expect("Failed to initialize DB");
        println!("✅ Database initialized");
    }

    let database_url = format!("sqlite://{}", db_path);
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    println!("✅ Database connected: {}", db_path);

    // Initialize Redis cache (Wave 4)
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
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

    // Initialize database schema with migrations
    db::init_database(&pool)
        .await
        .expect("Failed to initialize database schema");

    // Apply performance optimizations
    db::optimize_database(&pool)
        .await
        .expect("Failed to apply database optimizations");

    // Analyze tables for query optimization
    db::analyze_tables(&pool)
        .await
        .expect("Failed to analyze database tables");

    // Get and display database stats
    match db::get_database_stats(&pool).await {
        Ok(stats) => {
            println!("📊 Database Stats: {} pages × {} bytes = {:.2}MB",
                stats.page_count,
                stats.page_size,
                stats.total_size_bytes as f64 / (1024.0 * 1024.0)
            );
            println!("   Journal mode: {}", stats.journal_mode);
        }
        Err(e) => eprintln!("⚠️  Failed to get database stats: {}", e),
    }

    // Display migration history
    match db::get_migration_history(&pool).await {
        Ok(migrations) => {
            println!("📈 Migration History:");
            for m in migrations {
                println!("   [{}] {} ({}ms)", m.version, m.description, m.execution_time);
            }
        }
        Err(e) => eprintln!("⚠️  Failed to get migration history: {}", e),
    }

    println!("🔄 Starting server on http://{}:{}", settings.server.host, settings.server.port);

    // Initialize StorageBackend (Wave 3 - Local for dev, S3 for prod)
    let storage_backend: Arc<dyn storage::StorageBackend> = if std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        == "production"
    {
        // Production: S3/MinIO backend
        // TODO: Initialize S3StorageBackend from config
        Arc::new(storage::local_backend::LocalStorageBackend::new(
            &settings.upload.directory,
        ))
    } else {
        // Development: Local filesystem backend
        Arc::new(storage::local_backend::LocalStorageBackend::new(
            &settings.upload.directory,
        ))
    };
    println!("✅ StorageBackend initialized (Local)");

    // Create AppState with cached settings
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

// Root-level health check handlers
async fn health_check_handler() -> HttpResponse {
    use serde::Serialize;
    #[derive(Serialize)]
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
    match sqlx::query("SELECT 1").fetch_one(&app_state.db).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "database_connected",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
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

async fn initialize_db(db_path: &str) -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;

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
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            size INTEGER NOT NULL,
            path TEXT NOT NULL,
            uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Create test user: testuser / testpassword
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(b"testpassword", &salt)
        .map_err(|_| sqlx::Error::Configuration("Failed to hash password".into()))?
        .to_string();

    sqlx::query(
        "INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (?, ?, ?)"
    )
    .bind("test-user-1")
    .bind("testuser")
    .bind(&password_hash)
    .execute(&pool)
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
