use actix_web::{HttpResponse, web};
use chrono::Utc;
use futures::stream::Stream;
use parking_lot::RwLock;
use serde::Serialize;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::broadcast;

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

pub struct SseStream {
    rx: broadcast::Receiver<V2Event>,
}

impl Stream for SseStream {
    type Item = Result<actix_web::web::Bytes, actix_web::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    let sse = format!("data: {json}\n\n");
                    return Poll::Ready(Some(Ok(web::Bytes::from(sse))));
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    self.rx = self.rx.resubscribe();
                    let waker = cx.waker().clone();
                    let mut rx = self.rx.resubscribe();
                    tokio::spawn(async move {
                        let _ = rx.recv().await;
                        waker.wake();
                    });
                    return Poll::Pending;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    return Poll::Ready(None);
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    continue;
                }
            }
        }
    }
}

pub async fn subscribe_events(
    bus: web::Data<EventBus>,
) -> HttpResponse {
    let rx = bus.subscribe();
    let stream = SseStream { rx };
    HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .insert_header(("cache-control", "no-cache"))
        .insert_header(("connection", "keep-alive"))
        .streaming(stream)
}
