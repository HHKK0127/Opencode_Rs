use actix_web::{HttpResponse, web};
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::events::EventBus;
use crate::models::*;

type QuestionStore = Arc<RwLock<HashMap<String, QuestionV2Request>>>;

#[derive(Deserialize)]
pub struct SessionIdPath {
    id: String,
}

#[derive(Deserialize)]
pub struct QuestionRequestIdPath {
    id: String,
    request_id: String,
}

pub fn create_question_store() -> QuestionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn list_questions(
    path: web::Path<SessionIdPath>,
    store: web::Data<QuestionStore>,
) -> HttpResponse {
    let questions = store.read();
    let items: Vec<&QuestionV2Request> = questions.values()
        .filter(|q| q.session_id == path.id)
        .collect();
    HttpResponse::Ok().json(serde_json::json!({ "data": items }))
}

pub async fn reply_question(
    path: web::Path<QuestionRequestIdPath>,
    body: web::Json<QuestionReplyRequest>,
    store: web::Data<QuestionStore>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let mut questions = store.write();
    questions.remove(&path.request_id);

    crate::api::events::emit_event(&bus, "question.v2.replied", serde_json::json!({
        "sessionID": path.id,
        "requestID": path.request_id,
        "answers": body.answers,
    }));

    HttpResponse::NoContent().finish()
}

pub async fn reject_question(
    path: web::Path<QuestionRequestIdPath>,
    store: web::Data<QuestionStore>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let mut questions = store.write();
    questions.remove(&path.request_id);

    crate::api::events::emit_event(&bus, "question.v2.rejected", serde_json::json!({
        "sessionID": path.id,
        "requestID": path.request_id,
    }));

    HttpResponse::NoContent().finish()
}
