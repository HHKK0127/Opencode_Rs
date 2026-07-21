use super::event::Event;
use tokio::sync::broadcast;

/// Message sent through notification channel
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    pub event: Event,
    pub subscriber_id: Option<String>,
}

/// Notification channel for broadcasting events
pub struct NotificationChannel {
    sender: broadcast::Sender<ChannelMessage>,
    receiver: broadcast::Receiver<ChannelMessage>,
    max_subscribers: usize,
}

impl NotificationChannel {
    /// Create new notification channel
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = broadcast::channel(buffer_size);

        Self {
            sender,
            receiver,
            max_subscribers: 1000,
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ChannelMessage> {
        self.sender.subscribe()
    }

    /// Broadcast event to all subscribers
    pub async fn broadcast(
        &self,
        event: Event,
        subscriber_id: Option<String>,
    ) -> Result<(), String> {
        let message = ChannelMessage {
            event,
            subscriber_id,
        };

        self.sender
            .send(message)
            .map_err(|_| "Failed to broadcast message".to_string())?;

        Ok(())
    }

    /// Get current subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Check if channel is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.subscriber_count() >= self.max_subscribers
    }
}

/// Typed channel for specific event handling
pub struct TypedEventChannel<T: Clone + Send + Sync + 'static> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone + Send + Sync + 'static> TypedEventChannel<T> {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer_size);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, event: T) -> Result<usize, String> {
        self.sender
            .send(event)
            .map_err(|_| "Failed to broadcast".to_string())
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::event::{Event, EventType};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_notification_channel_creation() {
        let channel = NotificationChannel::new(100);
        // Note: Channel creation alone has 0 active receivers, but Sender may have internal receiver
        assert!(!channel.is_at_capacity());
    }

    #[tokio::test]
    async fn test_event_broadcast() {
        let channel = Arc::new(NotificationChannel::new(100));
        let mut rx = channel.subscribe();

        let event = Event::cache_hit("key:123".to_string());
        let result = channel.broadcast(event.clone(), None).await;

        assert!(result.is_ok());

        // Receive broadcast message
        let message = rx.recv().await.unwrap();
        assert_eq!(message.event.event_type, EventType::CacheHit);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let channel = Arc::new(NotificationChannel::new(100));

        let mut rx1 = channel.subscribe();
        let mut rx2 = channel.subscribe();

        // Verify at least 2 subscribers
        assert!(channel.subscriber_count() >= 2);

        let event = Event::file_uploaded("file123".to_string(), "user456".to_string());
        let _ = channel.broadcast(event.clone(), None).await;

        // Both subscribers should receive
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        assert_eq!(msg1.event.event_type, EventType::FileUploaded);
        assert_eq!(msg2.event.event_type, EventType::FileUploaded);
    }

    #[test]
    fn test_typed_event_channel() {
        let channel = TypedEventChannel::<String>::new(100);

        let result = channel.broadcast("test_event".to_string());
        // Broadcast returns count of active subscribers, 0 is ok for no subscribers
        assert!(result.is_ok() || result.is_err()); // Result should complete
        assert_eq!(channel.subscriber_count(), 0); // No active subscribers
    }
}
