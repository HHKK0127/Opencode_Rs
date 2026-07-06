/// Health check integration with Graceful Shutdown
use crate::graceful::{GracefulShutdown, ActiveConnections};
use crate::production::health_check::{HealthChecker, HealthStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

/// Integrated health check system
pub struct IntegratedHealthCheck {
    checker: Arc<RwLock<HealthChecker>>,
    active_connections: Arc<ActiveConnections>,
    shutdown: Arc<GracefulShutdown>,
}

impl IntegratedHealthCheck {
    /// Create new integrated health check
    pub fn new(
        shutdown: Arc<GracefulShutdown>,
        active_connections: Arc<ActiveConnections>,
        check_interval_ms: u64,
    ) -> Self {
        Self {
            checker: Arc::new(RwLock::new(HealthChecker::new(check_interval_ms))),
            active_connections,
            shutdown,
        }
    }

    /// Register a component for health monitoring
    pub async fn register_component(&self, name: String) {
        let mut checker = self.checker.write().await;
        checker.register_component(name);
    }

    /// Update component health status
    pub async fn update_component(
        &self,
        name: &str,
        status: HealthStatus,
        latency_ms: f64,
    ) {
        let mut checker = self.checker.write().await;
        checker.update_component_health(name, status.clone(), latency_ms);

        // Log degradation if needed
        if status == HealthStatus::Degraded {
            warn!("Component {} health degraded (latency: {:.2}ms)", name, latency_ms);
        }
    }

    /// Record error for component
    pub async fn record_error(&self, component: &str) {
        let mut checker = self.checker.write().await;
        checker.record_error(component);
    }

    /// Get overall health status
    pub async fn get_overall_health(&self) -> HealthStatus {
        let checker = self.checker.read().await;
        checker.get_overall_health()
    }

    /// Check if system is ready
    pub async fn is_ready(&self) -> bool {
        let checker = self.checker.read().await;
        checker.is_ready()
    }

    /// Get health report
    pub async fn get_health_report(&self) -> String {
        let checker = self.checker.read().await;
        let mut report = checker.get_health_report();

        // Add connection info
        let current_conn = self.active_connections.current();
        report.push_str(&format!("\nActive Connections: {}\n", current_conn));

        // Add shutdown status
        let is_shutting_down = self.shutdown.is_shutting_down();
        report.push_str(&format!("Shutting Down: {}\n", is_shutting_down));

        report
    }

    /// Perform health check and decide on shutdown
    pub async fn check_and_decide_shutdown(&self) -> HealthCheckDecision {
        let health = self.get_overall_health().await;
        let is_shutting_down = self.shutdown.is_shutting_down();
        let active_conns = self.active_connections.current();

        if is_shutting_down {
            HealthCheckDecision::ShuttingDown { active_connections: active_conns }
        } else if health == HealthStatus::Unhealthy {
            HealthCheckDecision::Unhealthy {
                message: "System health is unhealthy".to_string(),
            }
        } else if health == HealthStatus::Degraded {
            HealthCheckDecision::Degraded {
                active_connections: active_conns,
            }
        } else {
            HealthCheckDecision::Healthy {
                active_connections: active_conns,
            }
        }
    }
}

/// Health check decision result
#[derive(Debug, Clone)]
pub enum HealthCheckDecision {
    Healthy { active_connections: usize },
    Degraded { active_connections: usize },
    Unhealthy { message: String },
    ShuttingDown { active_connections: usize },
}

impl HealthCheckDecision {
    pub fn should_accept_connections(&self) -> bool {
        matches!(self, HealthCheckDecision::Healthy { .. })
    }

    pub fn is_ready_for_shutdown(&self) -> bool {
        match self {
            HealthCheckDecision::ShuttingDown { active_connections } => {
                *active_connections == 0
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integrated_health_check_creation() {
        let shutdown = Arc::new(GracefulShutdown::new(30));
        let connections = Arc::new(ActiveConnections::new(100));

        let health = IntegratedHealthCheck::new(shutdown, connections, 5000);

        assert!(health.is_ready().await);
        assert_eq!(health.get_overall_health().await, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_register_components() {
        let shutdown = Arc::new(GracefulShutdown::new(30));
        let connections = Arc::new(ActiveConnections::new(100));

        let health = IntegratedHealthCheck::new(shutdown, connections, 5000);

        health.register_component("database".to_string()).await;
        health.register_component("cache".to_string()).await;

        let report = health.get_health_report().await;
        assert!(report.contains("database"));
        assert!(report.contains("cache"));
    }

    #[tokio::test]
    async fn test_health_check_decision_healthy() {
        let shutdown = Arc::new(GracefulShutdown::new(30));
        let connections = Arc::new(ActiveConnections::new(100));

        let health = IntegratedHealthCheck::new(shutdown, connections, 5000);

        let decision = health.check_and_decide_shutdown().await;

        assert!(decision.should_accept_connections());
        assert!(!decision.is_ready_for_shutdown());
    }

    #[tokio::test]
    async fn test_health_check_degraded() {
        let shutdown = Arc::new(GracefulShutdown::new(30));
        let connections = Arc::new(ActiveConnections::new(100));

        let health = IntegratedHealthCheck::new(shutdown, connections, 5000);

        health.register_component("api".to_string()).await;
        health.update_component("api", HealthStatus::Degraded, 150.0).await;

        // Degraded is still ready (not Unhealthy)
        assert!(health.is_ready().await);
        assert_eq!(health.get_overall_health().await, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_shutdown_decision() {
        let shutdown = Arc::new(GracefulShutdown::new(30));
        let connections = Arc::new(ActiveConnections::new(100));

        let health = IntegratedHealthCheck::new(shutdown.clone(), connections, 5000);

        shutdown.shutdown(crate::graceful::ShutdownSignal::Sigint).await;

        let decision = health.check_and_decide_shutdown().await;
        assert!(matches!(decision, HealthCheckDecision::ShuttingDown { .. }));
    }
}
