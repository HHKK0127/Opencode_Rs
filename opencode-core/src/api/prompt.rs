use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::Deserialize;

use crate::api::events::{self, EventBus};
use crate::api::session::SessionStore;
use crate::models::PromptRequest;

#[derive(Deserialize)]
pub struct SessionIdPath {
    id: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PromptAsyncQuery {
    directory: Option<String>,
    workspace: Option<String>,
}

pub async fn prompt_v2(
    path: web::Path<SessionIdPath>,
    body: web::Json<PromptRequest>,
    store: web::Data<SessionStore>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let session_id = path.into_inner().id;
    let parts = body.parts.clone();
    let agent = body.agent.clone().unwrap_or_default();
    let model = body.model.clone();

    {
        let sessions = store.read();
        if !sessions.contains_key(&session_id) {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"}));
        }
    }

    let user_text = extract_text(&parts);
    let msg_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let user_msg = serde_json::json!({
        "id": msg_id,
        "type": "user",
        "text": user_text,
        "time": { "created": now },
    });

    {
        let mut sessions = store.write();
        if let Some(data) = sessions.get_mut(&session_id) {
            data.messages.push(user_msg);
            if let Some(ref mut info) = data.v2_info {
                info.time.updated = Utc::now().to_rfc3339();
            }
        }
    }

    let store_clone = store.clone();
    let bus_clone = bus.clone();
    let session_id_clone = session_id.clone();
    let model_val = model.map(|m| serde_json::to_value(m).unwrap_or_default());

    tokio::spawn(async move {
        process_prompt(session_id_clone, &store_clone, &bus_clone, &parts, &agent, model_val).await;
    });

    HttpResponse::NoContent().finish()
}

pub async fn prompt_async_v1(
    path: web::Path<SessionIdPath>,
    body: web::Json<serde_json::Value>,
    _query: web::Query<PromptAsyncQuery>,
    store: web::Data<SessionStore>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let session_id = path.into_inner().id;

    {
        let sessions = store.read();
        if !sessions.contains_key(&session_id) {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"}));
        }
    }

    let parts = body.get("parts").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    let agent = body.get("agent").and_then(|a| a.as_str()).unwrap_or("").to_string();
    let model_val = body.get("model").cloned();

    let user_text = extract_text(&parts);
    let msg_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let user_msg = serde_json::json!({
        "id": msg_id,
        "role": "user",
        "content": user_text,
        "created_at": now,
    });

    {
        let mut sessions = store.write();
        if let Some(data) = sessions.get_mut(&session_id) {
            data.messages.push(user_msg);
        }
    }

    let store_clone = store.clone();
    let bus_clone = bus.clone();

    tokio::spawn(async move {
        process_prompt(session_id, &store_clone, &bus_clone, &parts, &agent, model_val).await;
    });

    HttpResponse::NoContent().finish()
}

fn extract_text(parts: &[serde_json::Value]) -> String {
    parts.iter()
        .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn process_prompt(
    session_id: String,
    store: &SessionStore,
    bus: &EventBus,
    parts: &[serde_json::Value],
    agent: &str,
    _model: Option<serde_json::Value>,
) {
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    events::emit_event(bus, "session.next.agent.switched", serde_json::json!({
        "timestamp": now,
        "sessionID": session_id,
        "messageID": assistant_msg_id,
        "agent": if agent.is_empty() { "default" } else { agent },
    }));

    let model_info = serde_json::json!({
        "id": "claude-sonnet-4-20250514",
        "providerID": "anthropic",
    });

    events::emit_event(bus, "session.next.step.started", serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "sessionID": session_id,
        "assistantMessageID": assistant_msg_id,
        "agent": if agent.is_empty() { "default" } else { agent },
        "model": model_info,
    }));

    let text_id = uuid::Uuid::new_v4().to_string();
    let reply = generate_reply(parts);

    events::emit_event(bus, "session.next.text.started", serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "sessionID": session_id,
        "assistantMessageID": assistant_msg_id,
        "textID": text_id,
    }));

    let words: Vec<&str> = reply.split_whitespace().collect();
    let mut accumulated = String::new();
    for chunk in words.chunks(3) {
        let delta = chunk.join(" ") + " ";
        accumulated += &delta;

        events::emit_event(bus, "session.next.text.delta", serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "sessionID": session_id,
            "assistantMessageID": assistant_msg_id,
            "textID": text_id,
            "delta": delta,
        }));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    events::emit_event(bus, "session.next.text.ended", serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "sessionID": session_id,
        "assistantMessageID": assistant_msg_id,
        "textID": text_id,
        "text": accumulated.trim(),
    }));

    let end_time = Utc::now().to_rfc3339();

    events::emit_event(bus, "session.next.step.ended", serde_json::json!({
        "timestamp": end_time,
        "sessionID": session_id,
        "assistantMessageID": assistant_msg_id,
        "finish": "stop",
        "cost": 0.001,
        "tokens": {
            "input": 50,
            "output": accumulated.split_whitespace().count() as u64,
            "reasoning": 0,
            "cache": { "read": 0, "write": 0 },
        },
    }));

    let assistant_msg = serde_json::json!({
        "id": assistant_msg_id,
        "type": "assistant",
        "agent": if agent.is_empty() { "default" } else { agent },
        "model": model_info,
        "content": [{
            "type": "text",
            "id": text_id,
            "text": accumulated.trim(),
        }],
        "time": { "created": now, "completed": end_time },
        "finish": "stop",
        "cost": 0.001,
        "tokens": {
            "input": 50,
            "output": accumulated.split_whitespace().count() as u64,
        },
    });

    {
        let mut sessions = store.write();
        if let Some(data) = sessions.get_mut(&session_id) {
            data.messages.push(assistant_msg.clone());
            if let Some(ref mut info) = data.v2_info {
                info.time.updated = Utc::now().to_rfc3339();
                info.cost = 0.001;
            }
        }
    }

    events::emit_event(bus, "message.updated", serde_json::json!({
        "sessionID": session_id,
        "info": assistant_msg,
    }));
}

fn generate_reply(parts: &[serde_json::Value]) -> String {
    let user_text = extract_text(parts);
    if user_text.is_empty() {
        return "Hello! I'm the OpenCode Rust backend. How can I help you today?".to_string();
    }

    let lower = user_text.to_lowercase();
    if lower.contains("hello") || lower.contains("hi ") || lower == "hi" {
        return "Hello! How can I assist you with your development tasks today?".to_string();
    }
    if lower.contains("help") {
        return "I can help you with coding tasks, file operations, and more. What would you like me to work on?".to_string();
    }
    if lower.contains("?" ) {
        return format!("Great question! Let me work through that for you.\n\nBased on what you've asked, here's what I understand:\n\n> {}\n\nI'll help you figure this out. Let me analyze the situation and provide a comprehensive solution.\n\nIs there anything specific you'd like me to explain further?", user_text);
    }

    format!(
        "I understand you're asking about \"{}\". Let me help you with that.\n\n\
        I've analyzed your request and here's what I can do:\n\n\
        1. Process your request\n\
        2. Provide relevant information\n\
        3. Assist with any follow-up questions\n\n\
        Feel free to provide more details if you need something specific!",
        if user_text.len() > 50 { &user_text[..50] } else { &user_text }
    )
}
