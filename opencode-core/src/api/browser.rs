use actix_web::{web, HttpResponse};
use serde::Deserialize;
use validator::Validate;

use crate::api::events::EventBus;
use crate::browser::error::BrowserError;
use crate::browser::types::*;
use crate::browser::BrowserManager;

#[derive(Deserialize)]
pub struct SessionQuery {
    pub session: Option<String>,
}

fn session_name(query: &SessionQuery) -> String {
    query
        .session
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

fn map_error(e: BrowserError) -> HttpResponse {
    match e {
        BrowserError::ChromeNotFound | BrowserError::NotRunning => {
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"error": e.to_string()}))
        }
        BrowserError::NavigationTimeout { .. } => {
            HttpResponse::GatewayTimeout().json(serde_json::json!({"error": e.to_string()}))
        }
        BrowserError::TabNotFound { .. } | BrowserError::ElementNotFound { .. } => {
            HttpResponse::NotFound().json(serde_json::json!({"error": e.to_string()}))
        }
        BrowserError::PathTraversal(_) => {
            HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}))
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

// POST /api/browser/navigate
pub async fn navigate(
    browser: web::Data<BrowserManager>,
    body: web::Json<NavigateArgs>,
    query: web::Query<SessionQuery>,
    bus: web::Data<EventBus>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser
        .navigate(&session, body.into_inner(), bus.get_ref())
        .await
    {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/find_tab
pub async fn find_tab(
    browser: web::Data<BrowserManager>,
    body: web::Json<FindTabArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.find_tab(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// GET /api/browser/snapshot
pub async fn snapshot(
    browser: web::Data<BrowserManager>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    let session = session_name(&query);
    match browser.snapshot(&session).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/click
pub async fn click(
    browser: web::Data<BrowserManager>,
    body: web::Json<ClickArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.click(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/fill
pub async fn fill(
    browser: web::Data<BrowserManager>,
    body: web::Json<FillArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.fill(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/evaluate
pub async fn evaluate(
    browser: web::Data<BrowserManager>,
    body: web::Json<EvaluateArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.evaluate(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/screenshot
pub async fn screenshot(
    browser: web::Data<BrowserManager>,
    body: web::Json<ScreenshotArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.screenshot(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/save_as_pdf
pub async fn save_as_pdf(
    browser: web::Data<BrowserManager>,
    body: web::Json<SaveAsPdfArgs>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }
    let session = session_name(&query);
    match browser.save_as_pdf(&session, body.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// GET /api/browser/list_tabs
pub async fn list_tabs(
    browser: web::Data<BrowserManager>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    let session = session_name(&query);
    match browser.list_tabs(&session).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/close_tab
pub async fn close_tab(
    browser: web::Data<BrowserManager>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    let session = session_name(&query);
    match browser.close_tab(&session).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// POST /api/browser/close_session
pub async fn close_session(
    browser: web::Data<BrowserManager>,
    query: web::Query<SessionQuery>,
) -> HttpResponse {
    let session = session_name(&query);
    match browser.close_session(&session).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(e) => map_error(e),
    }
}

// GET /api/browser/status
pub async fn status(browser: web::Data<BrowserManager>) -> HttpResponse {
    HttpResponse::Ok().json(browser.status().await)
}
