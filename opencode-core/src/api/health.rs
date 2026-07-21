use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::time::Instant;

use super::events::EventBus;
use super::session::SessionStore;

pub struct HealthState {
    pub start_time: Instant,
    pub session_store: SessionStore,
    pub event_bus: EventBus,
}

#[derive(Serialize)]
struct HealthResponse {
    healthy: bool,
    version: String,
    uptime_secs: u64,
    components: ComponentsStatus,
}

#[derive(Serialize)]
struct ComponentsStatus {
    session_store: bool,
    event_bus: bool,
}

pub async fn global_health(state: web::Data<HealthState>) -> HttpResponse {
    let session_ok = state.session_store.try_read().is_some();
    let event_ok = true;

    let healthy = session_ok;
    let mut status = if healthy {
        HttpResponse::Ok()
    } else {
        HttpResponse::ServiceUnavailable()
    };

    status.json(HealthResponse {
        healthy,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        components: ComponentsStatus {
            session_store: session_ok,
            event_bus: event_ok,
        },
    })
}

pub async fn api_health(state: web::Data<HealthState>) -> HttpResponse {
    let session_ok = state.session_store.try_read().is_some();
    let event_ok = true;

    let healthy = session_ok;
    let mut status = if healthy {
        HttpResponse::Ok()
    } else {
        HttpResponse::ServiceUnavailable()
    };

    status.json(HealthResponse {
        healthy,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        components: ComponentsStatus {
            session_store: session_ok,
            event_bus: event_ok,
        },
    })
}
