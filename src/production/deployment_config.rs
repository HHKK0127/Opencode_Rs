use serde::{Deserialize, Serialize};

/// Deployment environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentEnvironment {
    Development,
    Staging,
    Production,
}

impl DeploymentEnvironment {
    pub fn as_str(&self) -> &str {
        match self {
            DeploymentEnvironment::Development => "development",
            DeploymentEnvironment::Staging => "staging",
            DeploymentEnvironment::Production => "production",
        }
    }
}

/// Production configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    pub environment: DeploymentEnvironment,
    pub replica_count: usize,
    pub max_connections: usize,
    pub enable_sentinel: bool,
    pub enable_persistence: bool,
    pub backup_interval_hours: u64,
    pub health_check_interval_ms: u64,
    pub graceful_shutdown_timeout_secs: u64,
}

impl ProductionConfig {
    pub fn new(environment: DeploymentEnvironment) -> Self {
        match environment {
            DeploymentEnvironment::Development => Self {
                environment,
                replica_count: 1,
                max_connections: 50,
                enable_sentinel: false,
                enable_persistence: false,
                backup_interval_hours: 24,
                health_check_interval_ms: 10000,
                graceful_shutdown_timeout_secs: 10,
            },
            DeploymentEnvironment::Staging => Self {
                environment,
                replica_count: 2,
                max_connections: 100,
                enable_sentinel: false,
                enable_persistence: true,
                backup_interval_hours: 12,
                health_check_interval_ms: 5000,
                graceful_shutdown_timeout_secs: 30,
            },
            DeploymentEnvironment::Production => Self {
                environment,
                replica_count: 3,
                max_connections: 500,
                enable_sentinel: true,
                enable_persistence: true,
                backup_interval_hours: 6,
                health_check_interval_ms: 1000,
                graceful_shutdown_timeout_secs: 60,
            },
        }
    }

    /// Get recommended worker count
    pub fn worker_count(&self) -> usize {
        match self.environment {
            DeploymentEnvironment::Development => 2,
            DeploymentEnvironment::Staging => 4,
            DeploymentEnvironment::Production => 8,
        }
    }

    /// Get recommended cache size
    pub fn cache_size_mb(&self) -> usize {
        match self.environment {
            DeploymentEnvironment::Development => 256,
            DeploymentEnvironment::Staging => 1024,
            DeploymentEnvironment::Production => 4096,
        }
    }

    /// Is high availability enabled
    pub fn is_ha_enabled(&self) -> bool {
        self.replica_count > 1 && self.enable_sentinel
    }
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self::new(DeploymentEnvironment::Production)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_creation() {
        let config = ProductionConfig::new(DeploymentEnvironment::Production);
        assert_eq!(config.environment, DeploymentEnvironment::Production);
        assert!(config.is_ha_enabled());
    }

    #[test]
    fn test_staging_config() {
        let config = ProductionConfig::new(DeploymentEnvironment::Staging);
        assert_eq!(config.replica_count, 2);
        assert!(config.enable_persistence);
        assert!(!config.is_ha_enabled()); // No Sentinel
    }

    #[test]
    fn test_worker_count_scaling() {
        let dev = ProductionConfig::new(DeploymentEnvironment::Development);
        let prod = ProductionConfig::new(DeploymentEnvironment::Production);

        assert!(dev.worker_count() < prod.worker_count());
    }

    #[test]
    fn test_cache_sizing() {
        let dev = ProductionConfig::new(DeploymentEnvironment::Development);
        let prod = ProductionConfig::new(DeploymentEnvironment::Production);

        assert!(dev.cache_size_mb() < prod.cache_size_mb());
    }
}
