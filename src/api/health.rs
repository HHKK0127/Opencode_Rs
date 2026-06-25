use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use chrono::Utc;
use std::time::Instant;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    pub database: bool,
    pub cache: bool,
}

#[derive(Serialize)]
struct ComponentHealth {
    status: &'static str,
    latency_ms: u64,
    message: Option<String>,
}

#[derive(Serialize)]
struct ReadinessResponse {
    ready: bool,
    timestamp: String,
    components: ReadinessComponents,
}

#[derive(Serialize)]
struct ReadinessComponents {
    database: ComponentHealth,
    cache: ComponentHealth,
}

#[derive(Serialize)]
struct LivenessResponse {
    alive: bool,
    timestamp: String,
    uptime_check: &'static str,
}

#[get("/health")]
pub async fn health_check(app_state: web::Data<AppState>) -> HttpResponse {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&app_state.db)
        .await
        .is_ok();

    let cache_ok = if let Some(cache) = &app_state.cache {
        cache.health_check().await.is_ok()
    } else {
        false
    };

    let status = if db_ok { "healthy" } else { "degraded" };

    HttpResponse::Ok().json(HealthStatus {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
        checks: HealthChecks {
            database: db_ok,
            cache: cache_ok,
        },
    })
}

/// Kubernetes readiness probe — returns 200 only when all components ready
#[get("/health/ready")]
pub async fn readiness_check(app_state: web::Data<AppState>) -> HttpResponse {
    // Database check with latency measurement
    let db_start = Instant::now();
    let db_result = sqlx::query("SELECT 1").fetch_one(&app_state.db).await;
    let db_latency = db_start.elapsed().as_millis() as u64;

    let db_health = match db_result {
        Ok(_) => ComponentHealth {
            status: "ok",
            latency_ms: db_latency,
            message: None,
        },
        Err(e) => ComponentHealth {
            status: "error",
            latency_ms: db_latency,
            message: Some(e.to_string()),
        },
    };

    // Redis/cache check with latency measurement
    let cache_start = Instant::now();
    let (cache_status, cache_msg) = if let Some(cache) = &app_state.cache {
        match cache.health_check().await {
            Ok(_) => ("ok", None),
            Err(e) => ("error", Some(e.to_string())),
        }
    } else {
        ("unavailable", Some("Redis not configured".to_string()))
    };
    let cache_latency = cache_start.elapsed().as_millis() as u64;

    let cache_health = ComponentHealth {
        status: cache_status,
        latency_ms: cache_latency,
        message: cache_msg,
    };

    let ready = db_health.status == "ok";

    let response = ReadinessResponse {
        ready,
        timestamp: Utc::now().to_rfc3339(),
        components: ReadinessComponents {
            database: db_health,
            cache: cache_health,
        },
    };

    if ready {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// Kubernetes liveness probe — returns 200 if process is alive
#[get("/health/live")]
pub async fn liveness_check() -> HttpResponse {
    HttpResponse::Ok().json(LivenessResponse {
        alive: true,
        timestamp: Utc::now().to_rfc3339(),
        uptime_check: "ok",
    })
}

#[get("/health/db")]
pub async fn db_health_check(app_state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query("SELECT 1").fetch_one(&app_state.db).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "database_connected",
            "timestamp": Utc::now().to_rfc3339()
        })),
        Err(e) => {
            log::error!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "database_unavailable",
                "error": e.to_string(),
                "timestamp": Utc::now().to_rfc3339()
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        .service(readiness_check)
        .service(liveness_check)
        .service(db_health_check);
}
