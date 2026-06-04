use actix_web::{get, web, HttpResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_at: String,
}

#[get("/projects")]
pub async fn list_projects() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "projects": vec![] as Vec<ProjectResponse>,
        "total": 0
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_projects);
}
