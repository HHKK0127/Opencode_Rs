use actix_web::{get, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct GlobalConfig {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    providers: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
struct ConfigResponse {
    providers: Vec<serde_json::Value>,
}

#[get("/global/config")]
pub async fn global_config() -> HttpResponse {
    HttpResponse::Ok().json(GlobalConfig {
        version: env!("CARGO_PKG_VERSION").to_string(),
        providers: None,
    })
}

#[get("/config")]
pub async fn get_config() -> HttpResponse {
    HttpResponse::Ok().json(ConfigResponse {
        providers: vec![],
    })
}

#[get("/config/providers")]
pub async fn config_providers() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "providers": []
    }))
}
