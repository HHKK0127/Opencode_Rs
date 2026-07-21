use actix_web::{web, HttpResponse};
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::events::EventBus;
use crate::models::*;

type PermissionStore = Arc<RwLock<HashMap<String, PermissionV2Request>>>;

#[derive(Deserialize)]
pub struct SessionIdPath {
    id: String,
}

#[derive(Deserialize)]
pub struct PermissionRequestIdPath {
    id: String,
    request_id: String,
}

pub fn create_permission_store() -> PermissionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn list_permissions(
    path: web::Path<SessionIdPath>,
    store: web::Data<PermissionStore>,
) -> HttpResponse {
    let permissions = store.read();
    let items: Vec<&PermissionV2Request> = permissions
        .values()
        .filter(|p| p.session_id == path.id)
        .collect();
    HttpResponse::Ok().json(serde_json::json!({ "data": items }))
}

pub async fn reply_permission(
    path: web::Path<PermissionRequestIdPath>,
    body: web::Json<PermissionReplyRequest>,
    store: web::Data<PermissionStore>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let mut permissions = store.write();
    permissions.remove(&path.request_id);

    crate::api::events::emit_event(
        &bus,
        "permission.v2.replied",
        serde_json::json!({
            "sessionID": path.id,
            "requestID": path.request_id,
            "reply": body.reply,
        }),
    );

    HttpResponse::NoContent().finish()
}
