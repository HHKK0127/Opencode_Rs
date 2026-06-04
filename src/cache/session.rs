use crate::cache::{RedisCache, CacheResult, CacheError};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use std::sync::Arc;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: Vec<String>,
}

impl SessionData {
    /// Create new session data
    pub fn new(user_id: String, username: String, permissions: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            username,
            created_at: now,
            last_activity: now,
            permissions,
        }
    }

    /// Check if session is still active
    pub fn is_active(&self) -> bool {
        let age = Utc::now() - self.last_activity;
        // Session valid if activity within last 24 hours
        age < Duration::hours(24)
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
}

/// Session manager for JWT + Redis
pub struct SessionManager {
    cache: Arc<RedisCache>,
}

impl SessionManager {
    pub fn new(cache: Arc<RedisCache>) -> Self {
        Self { cache }
    }

    /// Create a new session and store in Redis
    pub async fn create_session(
        &self,
        token: &str,
        user_id: &str,
        username: &str,
        permissions: Vec<String>,
    ) -> CacheResult<SessionData> {
        let session = SessionData::new(user_id.to_string(), username.to_string(), permissions);

        let cache_key = format!("session:{}", token);
        let ttl = std::time::Duration::from_secs(86400); // 24 hours

        self.cache.set(&cache_key, &session, Some(ttl)).await?;

        info!(
            "Session created: user={}, token_prefix={}...",
            username,
            &token[..std::cmp::min(8, token.len())]
        );

        Ok(session)
    }

    /// Validate and retrieve session from Redis
    pub async fn validate_session(&self, token: &str) -> CacheResult<SessionData> {
        let cache_key = format!("session:{}", token);

        let session: SessionData = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or_else(|| {
                debug!("Session not found or expired: {}", token);
                CacheError::KeyNotFound("Session not found".to_string())
            })?;

        if !session.is_active() {
            warn!("Session expired: user_id={}", session.user_id);
            self.cache.delete(&cache_key).await.ok();
            return Err(CacheError::KeyNotFound(
                "Session expired".to_string(),
            ));
        }

        debug!("Session validated: user={}", session.username);
        Ok(session)
    }

    /// Extend session TTL (refresh activity)
    pub async fn extend_session(&self, token: &str) -> CacheResult<()> {
        let cache_key = format!("session:{}", token);

        // Check if session exists
        if !self.cache.exists(&cache_key).await? {
            debug!("Cannot extend non-existent session: {}", token);
            return Err(CacheError::KeyNotFound(
                "Session not found".to_string(),
            ));
        }

        // Get current session and update activity
        let mut session: SessionData = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or_else(|| {
                CacheError::KeyNotFound("Session not found".to_string())
            })?;

        session.update_activity();

        // Store updated session with new TTL
        let ttl = std::time::Duration::from_secs(86400);
        self.cache.set(&cache_key, &session, Some(ttl)).await?;

        debug!("Session extended: user={}, token_prefix={}...",
            session.username,
            &token[..std::cmp::min(8, token.len())]
        );

        Ok(())
    }

    /// Invalidate session (logout)
    pub async fn invalidate_session(&self, token: &str) -> CacheResult<()> {
        let cache_key = format!("session:{}", token);
        self.cache.delete(&cache_key).await?;

        info!("Session invalidated: token_prefix={}...",
            &token[..std::cmp::min(8, token.len())]
        );

        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_info(&self, token: &str) -> CacheResult<Option<String>> {
        let cache_key = format!("session:{}", token);
        match self.cache.get::<SessionData>(&cache_key).await {
            Ok(Some(session)) => Ok(Some(format!(
                "User: {}, Created: {}, LastActivity: {}",
                session.username, session.created_at, session.last_activity
            ))),
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Error retrieving session info: {}", e);
                Err(e)
            }
        }
    }

    /// Check if user has specific permission
    pub async fn check_permission(
        &self,
        token: &str,
        permission: &str,
    ) -> CacheResult<bool> {
        let session = self.validate_session(token).await?;
        Ok(session.permissions.contains(&permission.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_creation() {
        let session = SessionData::new(
            "user123".to_string(),
            "john_doe".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert_eq!(session.user_id, "user123");
        assert_eq!(session.username, "john_doe");
        assert_eq!(session.permissions.len(), 2);
        assert!(session.is_active());
    }

    #[test]
    fn test_session_activity_update() {
        let mut session = SessionData::new(
            "user123".to_string(),
            "john_doe".to_string(),
            vec![],
        );

        let initial_activity = session.last_activity;
        session.update_activity();

        assert!(session.last_activity >= initial_activity);
    }

    #[test]
    fn test_session_active_check() {
        let session = SessionData::new(
            "user123".to_string(),
            "john_doe".to_string(),
            vec![],
        );

        // New session should be active
        assert!(session.is_active());
    }

    #[test]
    fn test_permission_check() {
        let session = SessionData::new(
            "user123".to_string(),
            "john_doe".to_string(),
            vec!["read".to_string(), "admin".to_string()],
        );

        assert!(session.permissions.contains(&"read".to_string()));
        assert!(session.permissions.contains(&"admin".to_string()));
        assert!(!session.permissions.contains(&"delete".to_string()));
    }

    #[test]
    fn test_session_serialization() {
        let session = SessionData::new(
            "user123".to_string(),
            "john_doe".to_string(),
            vec!["read".to_string()],
        );

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: SessionData = serde_json::from_str(&json).unwrap();

        assert_eq!(session.user_id, deserialized.user_id);
        assert_eq!(session.username, deserialized.username);
        assert_eq!(session.permissions, deserialized.permissions);
    }
}
