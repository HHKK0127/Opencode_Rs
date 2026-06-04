use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Replica status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReplicaStatus {
    Primary,
    Replica,
    Unhealthy,
    Recovering,
}

impl ReplicaStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ReplicaStatus::Primary => "primary",
            ReplicaStatus::Replica => "replica",
            ReplicaStatus::Unhealthy => "unhealthy",
            ReplicaStatus::Recovering => "recovering",
        }
    }
}

/// Replica information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    pub id: String,
    pub status: ReplicaStatus,
    pub lag_bytes: u64,
    pub last_heartbeat: u64,
}

impl ReplicaInfo {
    pub fn new(id: String) -> Self {
        Self {
            id,
            status: ReplicaStatus::Replica,
            lag_bytes: 0,
            last_heartbeat: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == ReplicaStatus::Primary || self.status == ReplicaStatus::Replica
    }
}

/// Failover manager
pub struct FailoverManager {
    replicas: HashMap<String, ReplicaInfo>,
    primary_id: Option<String>,
    failover_threshold_ms: u64,
}

impl FailoverManager {
    pub fn new(failover_threshold_ms: u64) -> Self {
        Self {
            replicas: HashMap::new(),
            primary_id: None,
            failover_threshold_ms,
        }
    }

    /// Register a replica
    pub fn register_replica(&mut self, id: String) {
        let mut replica = ReplicaInfo::new(id.clone());

        if self.primary_id.is_none() {
            replica.status = ReplicaStatus::Primary;
            self.primary_id = Some(id.clone());
        }

        self.replicas.insert(id, replica);
    }

    /// Update replica health
    pub fn update_replica_health(&mut self, id: &str, lag_bytes: u64, last_heartbeat: u64) {
        if let Some(replica) = self.replicas.get_mut(id) {
            replica.lag_bytes = lag_bytes;
            replica.last_heartbeat = last_heartbeat;
        }
    }

    /// Perform failover
    pub fn failover(&mut self, new_primary_id: String) -> Result<(), String> {
        if !self.replicas.contains_key(&new_primary_id) {
            return Err("Replica not found".to_string());
        }

        // Mark old primary as replica
        if let Some(old_primary_id) = &self.primary_id {
            if let Some(old_primary) = self.replicas.get_mut(old_primary_id) {
                old_primary.status = ReplicaStatus::Recovering;
            }
        }

        // Mark new primary
        if let Some(new_primary) = self.replicas.get_mut(&new_primary_id) {
            new_primary.status = ReplicaStatus::Primary;
        }

        self.primary_id = Some(new_primary_id);
        Ok(())
    }

    /// Get healthy replicas
    pub fn get_healthy_replicas(&self) -> Vec<ReplicaInfo> {
        self.replicas
            .values()
            .filter(|r| r.is_healthy())
            .cloned()
            .collect()
    }

    /// Get current primary
    pub fn get_primary(&self) -> Option<ReplicaInfo> {
        self.primary_id
            .as_ref()
            .and_then(|id| self.replicas.get(id).cloned())
    }

    /// Check if automatic failover needed
    pub fn should_failover(&self) -> bool {
        if let Some(primary) = self.get_primary() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let time_since_heartbeat = current_time - primary.last_heartbeat;
            time_since_heartbeat > self.failover_threshold_ms
        } else {
            false
        }
    }

    /// Get failover status
    pub fn get_status(&self) -> String {
        let healthy_count = self.get_healthy_replicas().len();
        let total_count = self.replicas.len();

        format!(
            "Replicas: {}/{} healthy, Primary: {:?}",
            healthy_count, total_count, self.primary_id
        )
    }
}

impl Default for FailoverManager {
    fn default() -> Self {
        Self::new(30000) // 30 second failover threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replica_registration() {
        let mut manager = FailoverManager::new(30000);
        manager.register_replica("primary".to_string());
        manager.register_replica("replica1".to_string());

        assert_eq!(manager.get_healthy_replicas().len(), 2);
        assert_eq!(manager.get_primary().unwrap().id, "primary");
    }

    #[test]
    fn test_failover() {
        let mut manager = FailoverManager::new(30000);
        manager.register_replica("primary".to_string());
        manager.register_replica("replica1".to_string());

        let result = manager.failover("replica1".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_primary().unwrap().id, "replica1");
    }

    #[test]
    fn test_invalid_failover() {
        let mut manager = FailoverManager::new(30000);
        manager.register_replica("primary".to_string());

        let result = manager.failover("nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_replica_health_tracking() {
        let mut manager = FailoverManager::new(30000);
        manager.register_replica("primary".to_string());
        manager.register_replica("replica1".to_string());

        manager.update_replica_health("replica1", 1024, 1000000);

        let replica = manager.replicas.get("replica1").unwrap();
        assert_eq!(replica.lag_bytes, 1024);
    }
}
