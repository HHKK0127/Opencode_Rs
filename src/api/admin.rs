use actix_web::{get, post, web, HttpResponse};
use serde_json::json;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::db;
use crate::db::MigrationCli;

/// Database health and optimization endpoint
/// Requires authentication
#[get("/admin/db/status")]
pub async fn db_status(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    match db::get_database_stats(&app_state.db).await {
        Ok(stats) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "healthy",
                "database": {
                    "size_mb": stats.total_size_bytes as f64 / (1024.0 * 1024.0),
                    "page_count": stats.page_count,
                    "page_size": stats.page_size,
                    "journal_mode": stats.journal_mode
                }
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Get migration history
/// Requires authentication
#[get("/admin/db/migrations")]
pub async fn migration_history(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    match db::get_migration_history(&app_state.db).await {
        Ok(migrations) => {
            let records: Vec<_> = migrations.iter().map(|m| {
                json!({
                    "version": m.version,
                    "description": m.description,
                    "installed_on": m.installed_on,
                    "execution_time_ms": m.execution_time,
                    "success": m.success
                })
            }).collect();

            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "migrations": records
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Analyze database tables (optimizes query performance)
/// Requires authentication
#[get("/admin/db/analyze")]
pub async fn analyze_database(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    match db::analyze_tables(&app_state.db).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Database analysis completed"
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Vacuum database (reclaim disk space)
/// Requires authentication
#[get("/admin/db/vacuum")]
pub async fn vacuum_database(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    match db::vacuum_database(&app_state.db).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Database vacuum completed"
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Run pending migrations using sqlx-cli
/// Requires authentication
#[post("/admin/db/migrate")]
pub async fn run_migrations(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let database_url = app_state.settings.database.url.clone();

    match MigrationCli::run_migrations(&database_url) {
        Ok(output) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Migrations completed successfully",
                "output": output
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Revert the last migration using sqlx-cli
/// Requires authentication
#[post("/admin/db/migrate/revert")]
pub async fn revert_migration(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let database_url = app_state.settings.database.url.clone();

    match MigrationCli::revert_migration(&database_url) {
        Ok(output) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Last migration reverted",
                "output": output
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Get migration information
/// Requires authentication
#[get("/admin/db/migrate/info")]
pub async fn migration_info(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let database_url = app_state.settings.database.url.clone();

    match MigrationCli::migration_info(&database_url) {
        Ok(info) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "information": info
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Validate all migrations
/// Requires authentication
#[get("/admin/db/migrate/validate")]
pub async fn validate_migrations(app_state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let database_url = app_state.settings.database.url.clone();

    match MigrationCli::validate_migrations(&database_url) {
        Ok(valid) => {
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "valid": valid,
                "message": if valid { "All migrations are valid" } else { "Some migrations have errors" }
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(db_status)
        .service(migration_history)
        .service(analyze_database)
        .service(vacuum_database)
        .service(run_migrations)
        .service(revert_migration)
        .service(migration_info)
        .service(validate_migrations);
}
