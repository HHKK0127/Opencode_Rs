use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Performance metrics
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    pub endpoint: String,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
}

impl OptimizationMetrics {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput_rps: 0.0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
        }
    }

    pub fn meets_slo(&self) -> bool {
        // SLO: p95 < 50ms, p99 < 200ms, error_rate < 0.1%, cache_hit > 80%
        self.p95_latency_ms < 50.0
            && self.p99_latency_ms < 200.0
            && self.error_rate < 0.001
            && self.cache_hit_rate > 0.8
    }

    pub fn optimization_needed(&self) -> bool {
        !self.meets_slo()
    }
}

/// Performance optimizer for endpoint analysis
pub struct PerformanceOptimizer {
    metrics: Arc<RwLock<HashMap<String, OptimizationMetrics>>>,
    optimization_strategies: Vec<String>,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            optimization_strategies: vec![
                "Enable caching".to_string(),
                "Add indexing".to_string(),
                "Batch processing".to_string(),
                "Connection pooling".to_string(),
                "Query optimization".to_string(),
            ],
        }
    }

    /// Record metrics for endpoint
    pub async fn record_metrics(&self, endpoint: String, metrics: OptimizationMetrics) {
        let mut m = self.metrics.write().await;
        m.insert(endpoint.clone(), metrics);
        debug!("Metrics recorded for endpoint: {}", endpoint);
    }

    /// Analyze metrics and suggest optimizations
    pub async fn analyze(&self, endpoint: &str) -> Vec<String> {
        let metrics = self.metrics.read().await;

        if let Some(m) = metrics.get(endpoint) {
            let mut suggestions = Vec::new();

            if m.avg_latency_ms > 10.0 {
                suggestions.push("Reduce latency: Enable caching".to_string());
            }

            if m.cache_hit_rate < 0.8 {
                suggestions.push("Improve cache hit rate: Review cache strategy".to_string());
            }

            if m.error_rate > 0.001 {
                suggestions.push("Reduce error rate: Add error handling".to_string());
            }

            if m.p99_latency_ms > 200.0 {
                suggestions.push("Optimize p99: Review query performance".to_string());
            }

            suggestions
        } else {
            Vec::new()
        }
    }

    /// Get optimization report
    pub async fn get_report(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut report = String::from("Performance Optimization Report\n");
        report.push_str("==================================\n\n");

        for (endpoint, m) in metrics.iter() {
            report.push_str(&format!("Endpoint: {}\n", endpoint));
            report.push_str(&format!("  Avg Latency: {:.2}ms\n", m.avg_latency_ms));
            report.push_str(&format!("  P95 Latency: {:.2}ms\n", m.p95_latency_ms));
            report.push_str(&format!("  P99 Latency: {:.2}ms\n", m.p99_latency_ms));
            report.push_str(&format!("  Throughput: {:.0} req/s\n", m.throughput_rps));
            report.push_str(&format!("  Error Rate: {:.2}%\n", m.error_rate * 100.0));
            report.push_str(&format!(
                "  Cache Hit Rate: {:.1}%\n",
                m.cache_hit_rate * 100.0
            ));
            report.push_str(&format!("  SLO Met: {}\n\n", m.meets_slo()));
        }

        report
    }

    /// Get optimization strategies
    pub fn get_strategies(&self) -> Vec<String> {
        self.optimization_strategies.clone()
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = OptimizationMetrics::new("/api/files".to_string());
        assert_eq!(metrics.endpoint, "/api/files");
        assert_eq!(metrics.avg_latency_ms, 0.0);
    }

    #[test]
    fn test_slo_validation() {
        let mut metrics = OptimizationMetrics::new("/api/files".to_string());
        metrics.p95_latency_ms = 30.0;
        metrics.p99_latency_ms = 100.0;
        metrics.error_rate = 0.0005;
        metrics.cache_hit_rate = 0.85;

        assert!(metrics.meets_slo());
    }

    #[test]
    fn test_optimization_needed() {
        let mut metrics = OptimizationMetrics::new("/api/files".to_string());
        metrics.p95_latency_ms = 100.0; // Over SLO
        metrics.cache_hit_rate = 0.5;

        assert!(metrics.optimization_needed());
    }

    #[tokio::test]
    async fn test_performance_optimizer() {
        let optimizer = PerformanceOptimizer::new();

        let metrics = OptimizationMetrics {
            endpoint: "/api/files".to_string(),
            avg_latency_ms: 15.0,
            p95_latency_ms: 45.0,
            p99_latency_ms: 150.0,
            throughput_rps: 1500.0,
            error_rate: 0.0005,
            cache_hit_rate: 0.82,
        };

        optimizer
            .record_metrics("/api/files".to_string(), metrics)
            .await;

        let strategies = optimizer.get_strategies();
        assert!(!strategies.is_empty());
    }
}
