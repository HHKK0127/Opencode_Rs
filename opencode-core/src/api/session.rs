use actix_web::{get, post, HttpResponse, web::Path};
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub type SessionStore = Arc<RwLock<HashMap<String, serde_json::Value>>>;

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
struct SessionIdPath {
    id: String,
}

pub fn create_store() -> SessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

#[get("/session")]
pub async fn list_sessions(store: actix_web::web::Data<SessionStore>) -> HttpResponse {
    let sessions = store.read().values().cloned().collect::<Vec<_>>();
    HttpResponse::Ok().json(SessionListResponse { sessions })
}

#[post("/session/{id}/init")]
pub async fn init_session(
    path: Path<SessionIdPath>,
    store: actix_web::web::Data<SessionStore>,
) -> HttpResponse {
    let session = serde_json::json!({
        "id": path.id,
        "status": "ok",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });
    store.write().insert(path.id.clone(), session.clone());
    HttpResponse::Ok().json(SessionStatusResponse {
        status: "ok".to_string(),
        session,
    })
}

#[get("/session/{id}")]
pub async fn get_session(
    path: Path<SessionIdPath>,
    store: actix_web::web::Data<SessionStore>,
) -> HttpResponse {
    let sessions = store.read();
    match sessions.get(&path.id) {
        Some(session) => HttpResponse::Ok().json(SessionStatusResponse {
            status: "ok".to_string(),
            session: session.clone(),
        }),
        None => HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"})),
    }
}

#[post("/session/{id}/abort")]
pub async fn abort_session(
    path: Path<SessionIdPath>,
    store: actix_web::web::Data<SessionStore>,
) -> HttpResponse {
    let mut sessions = store.write();
    if let Some(session) = sessions.get_mut(&path.id) {
        if let Some(obj) = session.as_object_mut() {
            obj.insert("status".to_string(), serde_json::json!("aborted"));
        }
        HttpResponse::Ok().json(SessionStatusResponse {
            status: "aborted".to_string(),
            session: session.clone(),
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "session not found"}))
    }
}

#[get("/session/{id}/todo")]
pub async fn session_todo() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"todos": []}))
}

#[get("/session/{id}/diff")]
pub async fn session_diff() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"diff": ""}))
}

#[get("/session/{id}/children")]
pub async fn session_children() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"children": []}))
}
