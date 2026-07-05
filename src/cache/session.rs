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

/// Upload session progress data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSessionData {
    pub session_id: String,
    pub file_id: Option<String>,
    pub user_id: String,
    pub total_size: i64,
    pub uploaded_size: i64,
    pub chunk_size: i64,
    pub status: String, // pending, uploading, completed, failed
    pub chunks_received: Vec<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UploadSessionData {
    pub fn new(
        session_id: String,
        user_id: String,
        total_size: i64,
        chunk_size: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            file_id: None,
            user_id,
            total_size,
            uploaded_size: 0,
            chunk_size,
            status: "pending".to_string(),
            chunks_received: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn progress_percent(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.uploaded_size as f32 / self.total_size as f32) * 100.0
        }
    }

    pub fn is_complete(&self) -> bool {
        self.uploaded_size >= self.total_size
    }
}

/// Upload session manager for Redis cache
pub struct UploadSessionManager {
    cache: Arc<RedisCache>,
}

impl UploadSessionManager {
    pub fn new(cache: Arc<RedisCache>) -> Self {
        Self { cache }
    }

    /// Create new upload session
    pub async fn create_session(
        &self,
        session_id: &str,
        user_id: &str,
        total_size: i64,
        chunk_size: i64,
    ) -> CacheResult<UploadSessionData> {
        let mut session = UploadSessionData::new(
            session_id.to_string(),
            user_id.to_string(),
            total_size,
            chunk_size,
        );
        session.status = "uploading".to_string();

        let cache_key = format!("upload_session:{}", session_id);
        let ttl = std::time::Duration::from_secs(86400); // 24 hours

        self.cache.set(&cache_key, &session, Some(ttl)).await?;

        info!(
            "Upload session created: session_id={}, size={} bytes",
            session_id, total_size
        );

        Ok(session)
    }

    /// Get upload session from cache
    pub async fn get_session(&self, session_id: &str) -> CacheResult<Option<UploadSessionData>> {
        let cache_key = format!("upload_session:{}", session_id);
        self.cache.get(&cache_key).await
    }

    /// Update session progress
    pub async fn update_progress(
        &self,
        session_id: &str,
        uploaded_size: i64,
        chunk_index: i32,
    ) -> CacheResult<UploadSessionData> {
        let cache_key = format!("upload_session:{}", session_id);

        let mut session: UploadSessionData = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or_else(|| {
                CacheError::KeyNotFound("Upload session not found".to_string())
            })?;

        session.uploaded_size = uploaded_size;
        if !session.chunks_received.contains(&chunk_index) {
            session.chunks_received.push(chunk_index);
        }
        session.updated_at = Utc::now();

        let ttl = std::time::Duration::from_secs(86400);
        self.cache.set(&cache_key, &session, Some(ttl)).await?;

        debug!(
            "Upload progress updated: session_id={}, progress={}%",
            session_id,
            session.progress_percent()
        );

        Ok(session)
    }

    /// Mark session as completed
    pub async fn mark_completed(
        &self,
        session_id: &str,
        file_id: &str,
    ) -> CacheResult<()> {
        let cache_key = format!("upload_session:{}", session_id);

        let mut session: UploadSessionData = self
            .cache
            .get(&cache_key)
            .await?
            .ok_or_else(|| {
                CacheError::KeyNotFound("Upload session not found".to_string())
            })?;

        session.file_id = Some(file_id.to_string());
        session.status = "completed".to_string();
        session.updated_at = Utc::now();

        let ttl = std::time::Duration::from_secs(86400);
        self.cache.set(&cache_key, &session, Some(ttl)).await?;

        info!(
            "Upload session completed: session_id={}, file_id={}",
            session_id, file_id
        );

        Ok(())
    }

    /// Delete upload session
    pub async fn delete_session(&self, session_id: &str) -> CacheResult<()> {
        let cache_key = format!("upload_session:{}", session_id);
        self.cache.delete(&cache_key).await?;

        info!("Upload session deleted: session_id={}", session_id);

        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &str) -> CacheResult<Option<String>> {
        let cache_key = format!("upload_session:{}", session_id);
        match self.cache.get::<UploadSessionData>(&cache_key).await {
            Ok(Some(session)) => Ok(Some(format!(
                "Progress: {:.1}% ({}/{} bytes), Chunks: {}, Status: {}",
                session.progress_percent(),
                session.uploaded_size,
                session.total_size,
                session.chunks_received.len(),
                session.status
            ))),
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Error retrieving session stats: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod upload_tests {
    use super::*;

    #[test]
    fn test_upload_session_creation() {
        let session = UploadSessionData::new(
            "sess123".to_string(),
            "user456".to_string(),
            1000000,
            102400,
        );

        assert_eq!(session.session_id, "sess123");
        assert_eq!(session.user_id, "user456");
        assert_eq!(session.uploaded_size, 0);
        assert_eq!(session.status, "pending");
        assert_eq!(session.progress_percent(), 0.0);
    }

    #[test]
    fn test_upload_progress_calculation() {
        let session = UploadSessionData {
            session_id: "sess".to_string(),
            file_id: None,
            user_id: "user".to_string(),
            total_size: 1000,
            uploaded_size: 500,
            chunk_size: 100,
            status: "uploading".to_string(),
            chunks_received: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(session.progress_percent(), 50.0);
        assert!(!session.is_complete());
    }

    #[test]
    fn test_upload_completion() {
        let mut session = UploadSessionData::new(
            "sess".to_string(),
            "user".to_string(),
            100,
            10,
        );

        session.uploaded_size = 100;
        assert!(session.is_complete());
    }
}
