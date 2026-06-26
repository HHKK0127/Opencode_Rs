use actix_web::{get, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct GlobalHealth {
    healthy: bool,
}

#[derive(Serialize)]
struct ApiHealth {
    healthy: bool,
    version: String,
}

#[get("/global/health")]
pub async fn global_health() -> HttpResponse {
    HttpResponse::Ok().json(GlobalHealth { healthy: true })
}

#[get("/api/health")]
pub async fn api_health() -> HttpResponse {
    HttpResponse::Ok().json(ApiHealth {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
