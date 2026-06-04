use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event types supported by the notification system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    FileUploaded,
    FileDeleted,
    FileDownloaded,
    SearchExecuted,
    UserLoggedIn,
    UserLoggedOut,
    SessionCreated,
    SessionExpired,
    CacheHit,
    CacheMiss,
    PerformanceAlert,
    SystemHealthCheck,
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::FileUploaded => "file.uploaded",
            EventType::FileDeleted => "file.deleted",
            EventType::FileDownloaded => "file.downloaded",
            EventType::SearchExecuted => "search.executed",
            EventType::UserLoggedIn => "user.logged_in",
            EventType::UserLoggedOut => "user.logged_out",
            EventType::SessionCreated => "session.created",
            EventType::SessionExpired => "session.expired",
            EventType::CacheHit => "cache.hit",
            EventType::CacheMiss => "cache.miss",
            EventType::PerformanceAlert => "performance.alert",
            EventType::SystemHealthCheck => "system.health_check",
        }
    }
}

/// Event data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub user_id: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: String,
    pub metadata: HashMap<String, String>,
}

impl EventData {
    pub fn new(action: String) -> Self {
        Self {
            user_id: None,
            resource_id: None,
            resource_type: None,
            action,
            metadata: HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_resource(mut self, resource_type: String, resource_id: String) -> Self {
        self.resource_type = Some(resource_type);
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Complete event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub event_type: EventType,
    pub data: EventData,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl Event {
    pub fn new(event_type: EventType, data: EventData, source: String) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            data,
            timestamp: Utc::now(),
            source,
        }
    }

    pub fn file_uploaded(file_id: String, user_id: String) -> Self {
        let data = EventData::new("upload".to_string())
            .with_user_id(user_id)
            .with_resource("file".to_string(), file_id);

        Self::new(EventType::FileUploaded, data, "file-api".to_string())
    }

    pub fn file_deleted(file_id: String, user_id: String) -> Self {
        let data = EventData::new("delete".to_string())
            .with_user_id(user_id)
            .with_resource("file".to_string(), file_id);

        Self::new(EventType::FileDeleted, data, "file-api".to_string())
    }

    pub fn cache_hit(cache_key: String) -> Self {
        let data = EventData::new("cache_hit".to_string())
            .with_metadata("cache_key".to_string(), cache_key);

        Self::new(EventType::CacheHit, data, "cache-layer".to_string())
    }

    pub fn user_logged_in(user_id: String) -> Self {
        let data = EventData::new("login".to_string())
            .with_user_id(user_id);

        Self::new(EventType::UserLoggedIn, data, "auth-service".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::FileUploaded.as_str(), "file.uploaded");
        assert_eq!(EventType::CacheHit.as_str(), "cache.hit");
    }

    #[test]
    fn test_event_data_building() {
        let data = EventData::new("test".to_string())
            .with_user_id("user123".to_string())
            .with_resource("file".to_string(), "file456".to_string());

        assert_eq!(data.user_id, Some("user123".to_string()));
        assert_eq!(data.resource_id, Some("file456".to_string()));
    }

    #[test]
    fn test_event_creation() {
        let event = Event::file_uploaded("file123".to_string(), "user456".to_string());

        assert_eq!(event.event_type, EventType::FileUploaded);
        assert_eq!(event.data.user_id, Some("user456".to_string()));
        assert_eq!(event.data.resource_id, Some("file123".to_string()));
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::cache_hit("metadata:123".to_string());
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.event_id, deserialized.event_id);
    }
}
