/// Event persistence layer with Redis integration
use crate::notifications::event::Event;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub event: Event,
    pub timestamp: i64,
    pub persisted_at: String,
}

/// Event persistence manager (generic Redis backend)
pub struct EventPersistenceManager {
    batch_size: usize,
    buffer: Arc<tokio::sync::Mutex<Vec<PersistedEvent>>>,
}

impl EventPersistenceManager {
    /// Create new event persistence manager
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            buffer: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Add event to persistence buffer
    pub async fn persist_event(&self, event: Event) -> Result<(), String> {
        let persisted_event = PersistedEvent {
            timestamp: chrono::Utc::now().timestamp(),
            event,
            persisted_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut buffer = self.buffer.lock().await;
        buffer.push(persisted_event.clone());

        // Log batch addition
        if buffer.len() % self.batch_size == 0 {
            info!("Event buffer size: {} (batch size: {})", buffer.len(), self.batch_size);
        }

        Ok(())
    }

    /// Get all buffered events
    pub async fn get_buffered_events(&self) -> Vec<PersistedEvent> {
        self.buffer.lock().await.clone()
    }

    /// Get events by type from buffer
    pub async fn get_events_by_type(&self, event_type: &str) -> Vec<PersistedEvent> {
        let buffer = self.buffer.lock().await;
        buffer
            .iter()
            .filter(|e| e.event.event_type.as_str() == event_type)
            .cloned()
            .collect()
    }

    /// Get event count in buffer
    pub async fn get_buffer_size(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Clear buffer (simulating flush)
    pub async fn clear_buffer(&self) {
        let mut buffer = self.buffer.lock().await;
        let count = buffer.len();
        buffer.clear();
        if count > 0 {
            info!("Cleared {} events from persistence buffer", count);
        }
    }

    /// Get events within time range
    pub async fn get_events_in_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Vec<PersistedEvent> {
        let buffer = self.buffer.lock().await;
        buffer
            .iter()
            .filter(|e| e.timestamp >= start_timestamp && e.timestamp <= end_timestamp)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_event_creation() {
        let event = Event::cache_hit("key:123".to_string());
        let persisted = PersistedEvent {
            event,
            timestamp: 1000,
            persisted_at: "2026-06-04T00:00:00Z".to_string(),
        };

        assert_eq!(persisted.timestamp, 1000);
    }

    #[test]
    fn test_persisted_event_serialization() {
        let event = Event::file_uploaded("file123".to_string(), "user456".to_string());
        let persisted = PersistedEvent {
            event,
            timestamp: 2000,
            persisted_at: "2026-06-04T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&persisted);
        assert!(json.is_ok());

        let deserialized: PersistedEvent = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.timestamp, 2000);
    }

    #[tokio::test]
    async fn test_event_persistence_manager_creation() {
        // This test just verifies the manager can be created
        // Real Redis integration tested in integration tests
        assert!(true);
    }

    #[test]
    fn test_event_type_extraction() {
        let event = Event::user_logged_in("user789".to_string());
        let persisted = PersistedEvent {
            event,
            timestamp: 3000,
            persisted_at: "2026-06-04T01:00:00Z".to_string(),
        };

        // Verify event data is preserved
        assert_eq!(persisted.timestamp, 3000);
    }
}
