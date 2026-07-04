use actix_web::{HttpResponse, web};
use tokio::sync::broadcast;
use serde::Serialize;
use std::sync::Arc;

pub type EventBus = Arc<broadcast::Sender<V2Event>>;

#[derive(Debug, Clone, Serialize)]
pub struct V2Event {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

pub fn create_event_bus() -> EventBus {
    Arc::new(broadcast::channel(2048).0)
}

pub fn emit_event(bus: &EventBus, event_type: &str, data: serde_json::Value) {
    let event = V2Event {
        id: uuid::Uuid::new_v4().to_string(),
        event_type: event_type.to_string(),
        data,
    };
    let _ = bus.send(event);
}

pub async fn subscribe_events(
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let rx = bus.subscribe();
    let stream = futures::stream::unfold(rx, |mut rx| async {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    let sse = format!("data: {json}\n\n");
                    return Some((Ok::<_, actix_web::Error>(web::Bytes::from(sse)), rx));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("SSE stream lagged by {} messages", n);
                    continue;
                }
            }
        }
    });

    HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .insert_header(("cache-control", "no-cache"))
        .insert_header(("connection", "keep-alive"))
        .streaming(stream)
}
