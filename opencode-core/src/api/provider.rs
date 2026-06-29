use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
struct ProviderListResponse {
    providers: Vec<serde_json::Value>,
}

pub async fn list_providers() -> HttpResponse {
    let providers = vec![
        serde_json::json!({
            "id": "anthropic",
            "name": "Anthropic",
            "type": "anthropic",
            "api_key_configured": false,
            "models": [
                {"id": "claude-sonnet-4-20250514", "name": "Claude Sonnet 4", "provider": "anthropic"},
                {"id": "claude-opus-4-20250514", "name": "Claude Opus 4", "provider": "anthropic"},
            ]
        }),
        serde_json::json!({
            "id": "openai",
            "name": "OpenAI",
            "type": "openai",
            "api_key_configured": false,
            "models": [
                {"id": "gpt-4o", "name": "GPT-4o", "provider": "openai"},
                {"id": "gpt-4o-mini", "name": "GPT-4o Mini", "provider": "openai"},
            ]
        }),
    ];
    HttpResponse::Ok().json(ProviderListResponse { providers })
}

pub async fn update_provider() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub async fn provider_oauth_authorize() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"url": ""}))
}

pub async fn provider_auth() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"providers": []}))
}
