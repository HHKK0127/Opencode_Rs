use actix_web::{web, HttpResponse};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::*;

pub struct SessionData {
    pub info: serde_json::Value,
    pub v2_info: Option<SessionV2Info>,
    pub messages: Vec<serde_json::Value>,
}

pub type SessionStore = Arc<RwLock<HashMap<String, SessionData>>>;

#[derive(Serialize)]
struct SessionListResponse {
    sessions: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct SessionStatusResponse {
    status: String,
    session: serde_json::Value,
}

#[derive(Deserialize)]
pub struct SessionIdPath {
    id: String,
}

pub fn create_store() -> SessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn list_sessions(store: web::Data<SessionStore>) -> HttpResponse {
    let sessions = store.read();
    let items: Vec<serde_json::Value> = sessions.values().map(|s| s.info.clone()).collect();
    drop(sessions);
    HttpResponse::Ok().json(SessionListResponse { sessions: items })
}

pub async fn init_session(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
) -> HttpResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let session = serde_json::json!({
        "id": path.id,
        "status": "ok",
        "created_at": now,
        "updated_at": now,
    });
    store.write().insert(
        path.id.clone(),
        SessionData {
            info: session.clone(),
            v2_info: None,
            messages: Vec::new(),
        },
    );
    HttpResponse::Ok().json(SessionStatusResponse {
        status: "ok".to_string(),
        session,
    })
}

pub async fn get_session(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
) -> HttpResponse {
    let sessions = store.read();
    match sessions.get(&path.id) {
        Some(data) => HttpResponse::Ok().json(SessionStatusResponse {
            status: "ok".to_string(),
            session: data.info.clone(),
        }),
        None => HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"})),
    }
}

pub async fn abort_session(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
) -> HttpResponse {
    let mut sessions = store.write();
    if let Some(data) = sessions.get_mut(&path.id) {
        if let Some(obj) = data.info.as_object_mut() {
            obj.insert("status".to_string(), serde_json::json!("aborted"));
        }
        HttpResponse::Ok().json(SessionStatusResponse {
            status: "aborted".to_string(),
            session: data.info.clone(),
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"}))
    }
}

pub async fn create_session_v2(
    body: web::Json<SessionCreateRequest>,
    store: web::Data<SessionStore>,
    bus: web::Data<crate::api::events::EventBus>,
) -> HttpResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let session_id = body
        .id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let location = body.location.clone().unwrap_or(LocationRef {
        directory: std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        workspace_id: None,
    });

    let v2_info = SessionV2Info {
        id: session_id.clone(),
        parent_id: None,
        project_id: "default".to_string(),
        agent: body.agent.clone(),
        model: body.model.clone(),
        cost: 0.0,
        tokens: TokenUsage {
            input: 0,
            output: 0,
            reasoning: 0,
            cache: CacheUsage { read: 0, write: 0 },
        },
        time: TimeInfo {
            created: now.clone(),
            updated: now.clone(),
            archived: None,
        },
        title: "New Session".to_string(),
        location,
        subpath: None,
    };

    let info_json = serde_json::to_value(&v2_info).unwrap_or_default();
    store.write().insert(
        session_id.clone(),
        SessionData {
            info: info_json.clone(),
            v2_info: Some(v2_info.clone()),
            messages: Vec::new(),
        },
    );

    crate::api::events::emit_event(
        &bus,
        "session.created",
        serde_json::json!({
            "sessionID": session_id,
            "info": info_json,
        }),
    );

    HttpResponse::Ok().json(serde_json::json!({ "data": v2_info }))
}

pub async fn get_session_v2(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
) -> HttpResponse {
    let sessions = store.read();
    match sessions.get(&path.id) {
        Some(data) => {
            if let Some(ref v2) = data.v2_info {
                HttpResponse::Ok().json(serde_json::json!({ "data": v2 }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({ "data": data.info }))
            }
        }
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "session not found" })),
    }
}

pub async fn delete_session_v2(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
    bus: web::Data<crate::api::events::EventBus>,
) -> HttpResponse {
    let mut sessions = store.write();
    if sessions.remove(&path.id).is_some() {
        crate::api::events::emit_event(
            &bus,
            "session.deleted",
            serde_json::json!({
                "sessionID": path.id,
            }),
        );
        HttpResponse::Ok().json(serde_json::json!({ "status": "deleted" }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "error": "session not found" }))
    }
}

pub async fn list_sessions_v2(store: web::Data<SessionStore>) -> HttpResponse {
    let sessions = store.read();
    let items: Vec<serde_json::Value> = sessions
        .values()
        .filter_map(|s| {
            s.v2_info
                .as_ref()
                .map(|v2| serde_json::to_value(v2).unwrap_or_default())
        })
        .collect();
    drop(sessions);
    HttpResponse::Ok().json(serde_json::json!({ "data": items }))
}

pub async fn get_session_messages(
    path: web::Path<SessionIdPath>,
    store: web::Data<SessionStore>,
    query: web::Query<MessagesQuery>,
) -> HttpResponse {
    let sessions = store.read();
    match sessions.get(&path.id) {
        Some(data) => {
            let mut msgs = data.messages.clone();
            let order = query.order.as_deref().unwrap_or("asc");
            if order == "desc" {
                msgs.reverse();
            }
            let limit = query.limit.unwrap_or(50).min(200);
            if msgs.len() > limit {
                msgs.truncate(limit);
            }
            HttpResponse::Ok().json(SessionMessagesResponse {
                data: msgs,
                cursor: MessagesCursor {
                    previous: None,
                    next: None,
                },
            })
        }
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "session not found" })),
    }
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub limit: Option<usize>,
    pub order: Option<String>,
    pub cursor: Option<String>,
}

pub async fn session_todo() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"todos": []}))
}

pub async fn session_diff() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"diff": ""}))
}

pub async fn session_children() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"children": []}))
}
