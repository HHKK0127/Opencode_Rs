use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alert rule for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric: String,
    pub threshold: f64,
    pub condition: AlertCondition,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
}

impl AlertRule {
    pub fn new(name: String, metric: String, threshold: f64, condition: AlertCondition) -> Self {
        Self {
            name,
            metric,
            threshold,
            condition,
            enabled: true,
        }
    }

    pub fn check(&self, value: f64) -> bool {
        match self.condition {
            AlertCondition::GreaterThan => value > self.threshold,
            AlertCondition::LessThan => value < self.threshold,
            AlertCondition::Equals => (value - self.threshold).abs() < 0.001,
        }
    }
}

/// Metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub timestamp: u64,
    pub value: f64,
}

pub struct MetricsCollector {
    metrics: HashMap<String, Vec<MetricValue>>,
    alert_rules: Vec<AlertRule>,
    max_retention: usize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            alert_rules: Vec::new(),
            max_retention: 1000,
        }
    }

    /// Record a metric value
    pub fn record_metric(&mut self, name: String, value: f64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metric = MetricValue { timestamp, value };

        let entry = self.metrics.entry(name).or_insert_with(Vec::new);
        entry.push(metric);

        // Keep only recent entries
        if entry.len() > self.max_retention {
            entry.drain(0..entry.len() - self.max_retention);
        }
    }

    /// Add alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }

    /// Get metric average
    pub fn get_average(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).and_then(|values| {
            if values.is_empty() {
                None
            } else {
                let sum: f64 = values.iter().map(|v| v.value).sum();
                Some(sum / values.len() as f64)
            }
        })
    }

    /// Get metric maximum
    pub fn get_maximum(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).and_then(|values| {
            values
                .iter()
                .map(|v| v.value)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        })
    }

    /// Check all alert rules
    pub fn check_alerts(&self) -> Vec<String> {
        let mut alerts = Vec::new();

        for rule in &self.alert_rules {
            if !rule.enabled {
                continue;
            }

            if let Some(avg) = self.get_average(&rule.metric) {
                if rule.check(avg) {
                    alerts.push(format!(
                        "ALERT: {} - {} = {:.2}",
                        rule.name, rule.metric, avg
                    ));
                }
            }
        }

        alerts
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();

        for (name, _) in &self.metrics {
            if let Some(avg) = self.get_average(name) {
                summary.insert(name.clone(), avg);
            }
        }

        summary
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule::new(
            "high_latency".to_string(),
            "p95_latency".to_string(),
            100.0,
            AlertCondition::GreaterThan,
        );

        assert!(rule.check(150.0));
        assert!(!rule.check(50.0));
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.record_metric("response_time".to_string(), 50.0);
        collector.record_metric("response_time".to_string(), 60.0);
        collector.record_metric("response_time".to_string(), 55.0);

        let avg = collector.get_average("response_time");
        assert!(avg.is_some());
        assert!(avg.unwrap() > 50.0 && avg.unwrap() < 60.0);
    }

    #[test]
    fn test_alert_detection() {
        let mut collector = MetricsCollector::new();

        let rule = AlertRule::new(
            "high_latency".to_string(),
            "latency".to_string(),
            100.0,
            AlertCondition::GreaterThan,
        );

        collector.add_alert_rule(rule);
        collector.record_metric("latency".to_string(), 150.0);

        let alerts = collector.check_alerts();
        assert!(!alerts.is_empty());
    }
}
