use actix_web::HttpResponse;
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

pub async fn global_config() -> HttpResponse {
    HttpResponse::Ok().json(GlobalConfig {
        version: env!("CARGO_PKG_VERSION").to_string(),
        providers: None,
    })
}

pub async fn get_config() -> HttpResponse {
    HttpResponse::Ok().json(ConfigResponse { providers: vec![] })
}

pub async fn config_providers() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "providers": []
    }))
}
