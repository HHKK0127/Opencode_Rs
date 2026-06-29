use actix_web::HttpResponse;
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

pub async fn global_health() -> HttpResponse {
    HttpResponse::Ok().json(GlobalHealth { healthy: true })
}

pub async fn api_health() -> HttpResponse {
    HttpResponse::Ok().json(ApiHealth {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
