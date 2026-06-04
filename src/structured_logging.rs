/// Structured logging integration with tracing
use tracing::{info, warn, error, debug};
use std::time::Instant;

/// Structured logging context
pub struct LogContext {
    request_id: String,
    user_id: Option<String>,
    component: String,
}

impl LogContext {
    /// Create new logging context
    pub fn new(component: String) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            component,
        }
    }

    /// Set user ID for logging
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Log request start
    pub fn log_request_start(&self, method: &str, path: &str) {
        info!(
            request_id = %self.request_id,
            user_id = ?self.user_id,
            component = %self.component,
            method = method,
            path = path,
            "Request started"
        );
    }

    /// Log request completion
    pub fn log_request_complete(&self, _method: &str, _path: &str, status: u16, elapsed_ms: u64) {
        if status >= 500 {
            error!(
                request_id = %self.request_id,
                user_id = ?self.user_id,
                status = status,
                elapsed_ms = elapsed_ms,
                "Request failed"
            );
        } else if status >= 400 {
            warn!(
                request_id = %self.request_id,
                status = status,
                elapsed_ms = elapsed_ms,
                "Request error"
            );
        } else {
            info!(
                request_id = %self.request_id,
                status = status,
                elapsed_ms = elapsed_ms,
                "Request completed"
            );
        }
    }

    /// Log operation with timing
    pub fn log_operation(&self, operation: &str, result: &str, elapsed_ms: u64) {
        if result == "success" {
            debug!(
                request_id = %self.request_id,
                component = %self.component,
                operation = operation,
                elapsed_ms = elapsed_ms,
                "Operation completed"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                component = %self.component,
                operation = operation,
                result = result,
                elapsed_ms = elapsed_ms,
                "Operation failed"
            );
        }
    }

    /// Log error event
    pub fn log_error(&self, error_code: &str, message: &str, details: Option<&str>) {
        error!(
            request_id = %self.request_id,
            component = %self.component,
            error_code = error_code,
            message = message,
            details = ?details,
            "Error occurred"
        );
    }

    /// Log database operation
    pub fn log_database_operation(
        &self,
        operation: &str,
        table: &str,
        rows_affected: u64,
        elapsed_ms: u64,
    ) {
        debug!(
            request_id = %self.request_id,
            operation = operation,
            table = table,
            rows_affected = rows_affected,
            elapsed_ms = elapsed_ms,
            "Database operation completed"
        );
    }

    /// Log cache operation
    pub fn log_cache_operation(
        &self,
        operation: &str,
        key: &str,
        hit: bool,
        elapsed_ms: u64,
    ) {
        debug!(
            request_id = %self.request_id,
            operation = operation,
            key = key,
            hit = hit,
            elapsed_ms = elapsed_ms,
            "Cache operation completed"
        );
    }

    /// Get request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Get component name
    pub fn component(&self) -> &str {
        &self.component
    }
}

/// Performance measurement utility
pub struct PerformanceTimer {
    start: Instant,
    label: String,
    context: String,
}

impl PerformanceTimer {
    /// Create new performance timer
    pub fn new(label: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
            context: context.into(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Log timing and return elapsed
    pub fn log(self) -> u64 {
        let elapsed_ms = self.elapsed_ms();
        if elapsed_ms > 100 {
            warn!(
                label = %self.label,
                context = %self.context,
                elapsed_ms = elapsed_ms,
                "Slow operation detected"
            );
        } else {
            debug!(
                label = %self.label,
                context = %self.context,
                elapsed_ms = elapsed_ms,
                "Operation timing"
            );
        }
        elapsed_ms
    }
}

/// System health event logging
pub struct HealthEventLogger;

impl HealthEventLogger {
    /// Log health check event
    pub fn log_health_check(component: &str, status: &str, latency_ms: f64) {
        info!(
            component = component,
            status = status,
            latency_ms = latency_ms,
            "Health check"
        );
    }

    /// Log connection change
    pub fn log_connection_change(event: &str, count: usize) {
        debug!(
            event = event,
            connection_count = count,
            "Connection event"
        );
    }

    /// Log shutdown event
    pub fn log_shutdown_event(phase: &str, message: &str) {
        info!(
            phase = phase,
            message = message,
            "Shutdown event"
        );
    }

    /// Log alert triggered
    pub fn log_alert(alert_name: &str, severity: &str, details: &str) {
        warn!(
            alert_name = alert_name,
            severity = severity,
            details = details,
            "Alert triggered"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        let ctx = LogContext::new("api".to_string());
        assert_eq!(ctx.component, "api");
        assert!(ctx.user_id.is_none());
    }

    #[test]
    fn test_log_context_with_user() {
        let ctx = LogContext::new("api".to_string())
            .with_user("user123".to_string());
        assert_eq!(ctx.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_request_id_generation() {
        let ctx1 = LogContext::new("service".to_string());
        let ctx2 = LogContext::new("service".to_string());

        // Each context should have unique request ID
        assert_ne!(ctx1.request_id(), ctx2.request_id());
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::new("test_op", "test_context");
        let elapsed = timer.log();
        assert!(elapsed >= 0);
    }

    #[test]
    fn test_performance_timer_elapsed() {
        let timer = PerformanceTimer::new("slow_op", "test");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 50);
    }

    #[test]
    fn test_log_context_methods() {
        let ctx = LogContext::new("database".to_string());

        // These methods should not panic
        ctx.log_request_start("POST", "/api/users");
        ctx.log_request_complete("POST", "/api/users", 201, 45);
        ctx.log_operation("insert_user", "success", 25);
        ctx.log_error("DB_ERR_001", "Connection failed", Some("Connection timeout"));
        ctx.log_database_operation("INSERT", "users", 1, 15);
        ctx.log_cache_operation("SET", "user:123", false, 5);
    }

    #[test]
    fn test_health_event_logger() {
        // These methods should not panic
        HealthEventLogger::log_health_check("database", "healthy", 12.5);
        HealthEventLogger::log_connection_change("increment", 42);
        HealthEventLogger::log_shutdown_event("phase_1", "Stopping new connections");
        HealthEventLogger::log_alert("high_cpu", "critical", "CPU usage > 90%");
    }
}
