use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

/// Tracks active connections
#[derive(Clone)]
pub struct ActiveConnections {
    count: Arc<AtomicUsize>,
    max_connections: usize,
}

impl ActiveConnections {
    /// Create new active connection tracker
    pub fn new(max_connections: usize) -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
            max_connections,
        }
    }

    /// Increment connection count
    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
        debug!("Connection incremented: {}", self.current());
    }

    /// Decrement connection count
    pub fn decrement(&self) {
        self.count.fetch_sub(1, Ordering::Relaxed);
        debug!("Connection decremented: {}", self.current());
    }

    /// Get current connection count
    pub fn current(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Wait for all connections to complete
    pub async fn wait_for_completion(&self, timeout_secs: u64) {
        let start = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        while self.current() > 0 {
            if start.elapsed() > timeout_duration {
                info!(
                    "Connection completion timeout: {} connections still active",
                    self.current()
                );
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if self.current() == 0 {
            info!("All connections completed");
        }
    }

    /// Check if at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.current() >= self.max_connections
    }

    /// Try to register a new connection
    pub fn try_register(&self) -> Result<ConnectionGuard, String> {
        if self.is_at_capacity() {
            return Err("Connection limit reached".to_string());
        }

        self.increment();
        Ok(ConnectionGuard {
            connections: self.clone(),
        })
    }
}

/// RAII guard for connection tracking
pub struct ConnectionGuard {
    connections: ActiveConnections,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.connections.decrement();
    }
}

/// Connection manager for graceful shutdown
pub struct ConnectionManager {
    active_connections: ActiveConnections,
    max_connections: usize,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new(max_connections: usize) -> Self {
        Self {
            active_connections: ActiveConnections::new(max_connections),
            max_connections,
        }
    }

    /// Get active connections tracker
    pub fn get_active_connections(&self) -> ActiveConnections {
        self.active_connections.clone()
    }

    /// Get current connection count
    pub fn current_count(&self) -> usize {
        self.active_connections.current()
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> usize {
        self.max_connections - self.current_count()
    }

    /// Wait for all connections to close
    pub async fn wait_all_closed(&self, timeout_secs: u64) {
        self.active_connections.wait_for_completion(timeout_secs).await;
    }

    /// Check health of connection system
    pub fn health_check(&self) -> ConnectionHealth {
        let current = self.current_count();
        let utilization = (current as f64 / self.max_connections as f64) * 100.0;

        ConnectionHealth {
            current_connections: current,
            max_connections: self.max_connections,
            utilization_percent: utilization,
            at_capacity: self.active_connections.is_at_capacity(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub current_connections: usize,
    pub max_connections: usize,
    pub utilization_percent: f64,
    pub at_capacity: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_connections_creation() {
        let conn = ActiveConnections::new(100);
        assert_eq!(conn.current(), 0);
    }

    #[test]
    fn test_connection_increment_decrement() {
        let conn = ActiveConnections::new(100);
        conn.increment();
        assert_eq!(conn.current(), 1);
        conn.increment();
        assert_eq!(conn.current(), 2);
        conn.decrement();
        assert_eq!(conn.current(), 1);
    }

    #[test]
    fn test_connection_guard() {
        let conn = ActiveConnections::new(2);
        let guard1 = conn.try_register();
        assert!(guard1.is_ok());
        assert_eq!(conn.current(), 1);

        let guard2 = conn.try_register();
        assert!(guard2.is_ok());
        assert_eq!(conn.current(), 2);

        // At capacity
        assert!(conn.is_at_capacity());

        drop(guard1);
        assert_eq!(conn.current(), 1);
    }

    #[test]
    fn test_connection_manager_creation() {
        let manager = ConnectionManager::new(500);
        assert_eq!(manager.max_connections, 500);
        assert_eq!(manager.current_count(), 0);
    }

    #[test]
    fn test_connection_manager_capacity() {
        let manager = ConnectionManager::new(100);
        assert_eq!(manager.remaining_capacity(), 100);

        let active = manager.get_active_connections();
        active.increment();
        active.increment();
        active.increment();

        assert_eq!(manager.current_count(), 3);
        assert_eq!(manager.remaining_capacity(), 97);
    }

    #[test]
    fn test_connection_health() {
        let manager = ConnectionManager::new(1000);
        let active = manager.get_active_connections();

        // Add 500 connections (50% utilization)
        for _ in 0..500 {
            active.increment();
        }

        let health = manager.health_check();
        assert_eq!(health.current_connections, 500);
        assert_eq!(health.max_connections, 1000);
        assert_eq!(health.utilization_percent, 50.0);
    }

    #[tokio::test]
    async fn test_wait_for_completion() {
        let manager = ConnectionManager::new(100);
        let active = manager.get_active_connections();

        active.increment();
        active.increment();

        // Spawn task to decrement after 100ms
        let active_clone = active.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            active_clone.decrement();
            active_clone.decrement();
        });

        manager.wait_all_closed(5).await;
        assert_eq!(manager.current_count(), 0);
    }
}
