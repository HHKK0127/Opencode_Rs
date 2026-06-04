use actix_web::{get, HttpResponse};
use crate::middleware::metrics::get_metrics;

#[get("/metrics")]
pub async fn expose_metrics() -> HttpResponse {
    match get_metrics() {
        Ok(metrics_text) => {
            HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .body(metrics_text)
        }
        Err(e) => {
            log::error!("Failed to generate metrics: {}", e);
            HttpResponse::InternalServerError()
                .body("Failed to generate metrics")
        }
    }
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(expose_metrics);
}
