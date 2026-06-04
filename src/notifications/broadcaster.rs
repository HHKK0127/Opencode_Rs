use super::event::{Event, EventType};
use super::channel::NotificationChannel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Broadcaster configuration
#[derive(Debug, Clone)]
pub struct BroadcasterConfig {
    pub channel_buffer_size: usize,
    pub max_event_history: usize,
}

impl Default for BroadcasterConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 1000,
            max_event_history: 10000,
        }
    }
}

/// Event broadcaster with subscriber management
pub struct EventBroadcaster {
    channel: NotificationChannel,
    event_history: Arc<RwLock<Vec<Event>>>,
    subscriber_filters: Arc<RwLock<HashMap<String, Vec<EventType>>>>,
    config: BroadcasterConfig,
}

impl EventBroadcaster {
    /// Create new event broadcaster
    pub fn new(config: BroadcasterConfig) -> Self {
        Self {
            channel: NotificationChannel::new(config.channel_buffer_size),
            event_history: Arc::new(RwLock::new(Vec::new())),
            subscriber_filters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Broadcast event to all subscribers
    pub async fn broadcast(&self, event: Event) -> Result<(), String> {
        info!("Broadcasting event: {:?}", event.event_type);

        // Add to history
        let mut history = self.event_history.write().await;
        history.push(event.clone());

        // Keep history size under control
        if history.len() > self.config.max_event_history {
            let to_remove = history.len() - self.config.max_event_history;
            history.drain(0..to_remove);
        }

        // Broadcast to channel
        self.channel.broadcast(event, None).await?;

        Ok(())
    }

    /// Subscribe to specific event types
    pub async fn subscribe(&self, subscriber_id: String, event_types: Vec<EventType>) {
        let mut filters = self.subscriber_filters.write().await;
        filters.insert(subscriber_id, event_types);

        debug!("Subscriber registered with event filters");
    }

    /// Unsubscribe subscriber
    pub async fn unsubscribe(&self, subscriber_id: &str) {
        let mut filters = self.subscriber_filters.write().await;
        filters.remove(subscriber_id);

        debug!("Subscriber unregistered: {}", subscriber_id);
    }

    /// Get event history
    pub async fn get_history(&self, limit: usize) -> Vec<Event> {
        let history = self.event_history.read().await;
        history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: EventType) -> Vec<Event> {
        let history = self.event_history.read().await;
        history
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get subscriber count
    pub fn get_subscriber_count(&self) -> usize {
        self.channel.subscriber_count()
    }

    /// Get event history size
    pub async fn get_history_size(&self) -> usize {
        self.event_history.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());
        // Broadcaster may have internal subscribers, just verify it's created
        assert!(broadcaster.get_history_size().await == 0);
    }

    #[tokio::test]
    async fn test_event_broadcast() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());

        let event = Event::cache_hit("key:123".to_string());
        let result = broadcaster.broadcast(event.clone()).await;

        assert!(result.is_ok());
        assert_eq!(broadcaster.get_history_size().await, 1);
    }

    #[tokio::test]
    async fn test_subscriber_registration() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());

        let event_types = vec![EventType::FileUploaded, EventType::CacheHit];
        broadcaster.subscribe("sub:001".to_string(), event_types).await;

        let filters = &broadcaster.subscriber_filters.read().await;
        assert!(filters.contains_key("sub:001"));
    }

    #[tokio::test]
    async fn test_event_history() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());

        for i in 0..5 {
            let event = Event::cache_hit(format!("key:{}", i));
            let _ = broadcaster.broadcast(event).await;
        }

        let history = broadcaster.get_history(10).await;
        assert_eq!(history.len(), 5);
    }

    #[tokio::test]
    async fn test_get_events_by_type() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());

        // Broadcast different event types
        let cache_hit = Event::cache_hit("key:123".to_string());
        let file_upload = Event::file_uploaded("file:456".to_string(), "user:789".to_string());

        let _ = broadcaster.broadcast(cache_hit).await;
        let _ = broadcaster.broadcast(file_upload).await;

        // Get events by type
        let cache_hits = broadcaster.get_events_by_type(EventType::CacheHit).await;
        let uploads = broadcaster.get_events_by_type(EventType::FileUploaded).await;

        assert_eq!(cache_hits.len(), 1);
        assert_eq!(uploads.len(), 1);
    }

    #[tokio::test]
    async fn test_subscriber_unregistration() {
        let broadcaster = EventBroadcaster::new(BroadcasterConfig::default());

        broadcaster.subscribe("sub:001".to_string(), vec![EventType::CacheHit]).await;
        assert_eq!(broadcaster.subscriber_filters.read().await.len(), 1);

        broadcaster.unsubscribe("sub:001").await;
        assert_eq!(broadcaster.subscriber_filters.read().await.len(), 0);
    }
}
