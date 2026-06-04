use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use chrono::Utc;

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
}

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now().to_rfc3339(),
        checks: HealthChecks { database: true },
    })
}

#[get("/health/db")]
pub async fn db_health_check(app_state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query("SELECT 1").fetch_one(&app_state.db).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "database_connected",
                "timestamp": Utc::now().to_rfc3339()
            }))
        }
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
    cfg.service(health_check).service(db_health_check);
}
