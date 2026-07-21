use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn as_str(&self) -> &str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
        }
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: f64,
    pub error_count: u64,
}

impl ComponentHealth {
    pub fn new(name: String) -> Self {
        Self {
            name,
            status: HealthStatus::Healthy,
            latency_ms: 0.0,
            error_count: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
}

/// System health checker
pub struct HealthChecker {
    components: HashMap<String, ComponentHealth>,
    check_interval_ms: u64,
}

impl HealthChecker {
    pub fn new(check_interval_ms: u64) -> Self {
        Self {
            components: HashMap::new(),
            check_interval_ms,
        }
    }

    /// Register a component for health checks
    pub fn register_component(&mut self, name: String) {
        self.components
            .insert(name.clone(), ComponentHealth::new(name.clone()));
        debug!("Component registered: {}", name);
    }

    /// Update component health
    pub fn update_component_health(&mut self, name: &str, status: HealthStatus, latency_ms: f64) {
        if let Some(component) = self.components.get_mut(name) {
            component.status = status.clone();
            component.latency_ms = latency_ms;

            if status != HealthStatus::Healthy {
                warn!("Component {} health degraded: {:?}", name, status);
            }
        }
    }

    /// Record error for component
    pub fn record_error(&mut self, name: &str) {
        if let Some(component) = self.components.get_mut(name) {
            component.error_count += 1;

            if component.error_count > 10 {
                component.status = HealthStatus::Unhealthy;
            } else if component.error_count > 3 {
                component.status = HealthStatus::Degraded;
            }
        }
    }

    /// Get overall system health
    pub fn get_overall_health(&self) -> HealthStatus {
        let unhealthy_count = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();

        let degraded_count = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > self.components.len() / 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get health report
    pub fn get_health_report(&self) -> String {
        let overall = self.get_overall_health();
        let mut report = format!("System Health: {}\n", overall.as_str());
        report.push_str("Components:\n");

        for (name, health) in &self.components {
            report.push_str(&format!(
                "  {}: {} (latency: {:.2}ms, errors: {})\n",
                name,
                health.status.as_str(),
                health.latency_ms,
                health.error_count
            ));
        }

        report
    }

    /// Get readiness status
    pub fn is_ready(&self) -> bool {
        self.get_overall_health() != HealthStatus::Unhealthy
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(5000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health() {
        let health = ComponentHealth::new("api".to_string());
        assert!(health.is_healthy());
    }

    #[test]
    fn test_health_checker() {
        let mut checker = HealthChecker::new(5000);
        checker.register_component("database".to_string());
        checker.register_component("cache".to_string());

        checker.update_component_health("database", HealthStatus::Healthy, 5.0);
        checker.update_component_health("cache", HealthStatus::Healthy, 2.0);

        assert_eq!(checker.get_overall_health(), HealthStatus::Healthy);
        assert!(checker.is_ready());
    }

    #[test]
    fn test_degraded_health() {
        let mut checker = HealthChecker::new(5000);
        checker.register_component("api".to_string());

        checker.update_component_health("api", HealthStatus::Degraded, 100.0);

        assert_eq!(checker.get_overall_health(), HealthStatus::Degraded);
    }

    #[test]
    fn test_unhealthy_detection() {
        let mut checker = HealthChecker::new(5000);
        checker.register_component("db".to_string());

        for _ in 0..15 {
            checker.record_error("db");
        }

        assert_eq!(checker.get_overall_health(), HealthStatus::Unhealthy);
    }
}
