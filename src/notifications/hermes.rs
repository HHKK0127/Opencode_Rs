use super::event::Event;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub event_type: String,
    pub count: u64,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
}

/// Analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub report_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u64,
    pub event_stats: HashMap<String, EventStats>,
    pub top_users: Vec<(String, u64)>,
    pub performance_metrics: HashMap<String, f64>,
}

impl AnalyticsReport {
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            report_id: uuid::Uuid::new_v4().to_string(),
            period_start,
            period_end,
            total_events: 0,
            event_stats: HashMap::new(),
            top_users: Vec::new(),
            performance_metrics: HashMap::new(),
        }
    }
}

/// Hermes Analytics System
pub struct HermesAnalytics {
    events: Arc<RwLock<Vec<Event>>>,
    statistics: Arc<RwLock<HashMap<String, u64>>>,
    user_activity: Arc<RwLock<HashMap<String, u64>>>,
}

impl Default for HermesAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

impl HermesAnalytics {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(HashMap::new())),
            user_activity: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record event for analytics
    pub async fn record_event(&self, event: Event) {
        let event_type = event.event_type.as_str().to_string();

        // Add to events
        self.events.write().await.push(event.clone());

        // Update statistics
        let mut stats = self.statistics.write().await;
        *stats.entry(event_type).or_insert(0) += 1;

        // Track user activity
        if let Some(user_id) = &event.data.user_id {
            let mut activity = self.user_activity.write().await;
            *activity.entry(user_id.clone()).or_insert(0) += 1;
        }

        debug!("Event recorded for analytics: {:?}", event.event_type);
    }

    /// Get statistics by event type
    pub async fn get_event_statistics(&self) -> HashMap<String, u64> {
        self.statistics.read().await.clone()
    }

    /// Get top users by activity
    pub async fn get_top_users(&self, limit: usize) -> Vec<(String, u64)> {
        let activity = self.user_activity.read().await;
        let mut users: Vec<_> = activity
            .iter()
            .map(|(user, count)| (user.clone(), *count))
            .collect();

        users.sort_by_key(|a| std::cmp::Reverse(a.1));
        users.into_iter().take(limit).collect()
    }

    /// Generate analytics report
    pub async fn generate_report(&self, period_hours: i64) -> AnalyticsReport {
        let now = Utc::now();
        let period_start = now - Duration::hours(period_hours);

        let events = self.events.read().await;
        let filtered_events: Vec<_> = events
            .iter()
            .filter(|e| e.timestamp >= period_start && e.timestamp <= now)
            .cloned()
            .collect();

        let mut report = AnalyticsReport::new(period_start, now);
        report.total_events = filtered_events.len() as u64;

        // Build event statistics
        for event in &filtered_events {
            let event_type = event.event_type.as_str().to_string();
            report
                .event_stats
                .entry(event_type)
                .or_insert_with(|| EventStats {
                    event_type: event.event_type.as_str().to_string(),
                    count: 0,
                    first_occurrence: event.timestamp,
                    last_occurrence: event.timestamp,
                })
                .count += 1;
        }

        // Update last occurrence
        for event in &filtered_events {
            let event_type = event.event_type.as_str().to_string();
            if let Some(stat) = report.event_stats.get_mut(&event_type) {
                stat.last_occurrence = event.timestamp;
            }
        }

        // Get top users
        let activity = self.user_activity.read().await;
        let mut top_users: Vec<_> = activity
            .iter()
            .map(|(user, count)| (user.clone(), *count))
            .collect();
        top_users.sort_by_key(|a| std::cmp::Reverse(a.1));
        report.top_users = top_users.into_iter().take(10).collect();

        // Add performance metrics
        if !filtered_events.is_empty() {
            report.performance_metrics.insert(
                "events_per_hour".to_string(),
                filtered_events.len() as f64 / period_hours as f64,
            );
            report
                .performance_metrics
                .insert("total_users".to_string(), activity.len() as f64);
        }

        info!(
            "Analytics report generated: {} events in {} hours",
            report.total_events, period_hours
        );
        report
    }

    /// Get event count
    pub async fn get_event_count(&self) -> usize {
        self.events.read().await.len()
    }

    /// Clear old events (retention policy)
    pub async fn cleanup_old_events(&self, days: i64) {
        let cutoff = Utc::now() - Duration::days(days);
        let mut events = self.events.write().await;
        events.retain(|e| e.timestamp >= cutoff);

        debug!("Cleaned up events older than {} days", days);
    }
}

/// Statistics collector for aggregating metrics
pub struct StatisticsCollector {
    hermes: Arc<HermesAnalytics>,
}

impl StatisticsCollector {
    pub fn new(hermes: Arc<HermesAnalytics>) -> Self {
        Self { hermes }
    }

    /// Record event
    pub async fn record(&self, event: Event) {
        self.hermes.record_event(event).await;
    }

    /// Get summary statistics
    pub async fn get_summary(&self) -> HashMap<String, u64> {
        self.hermes.get_event_statistics().await
    }

    /// Get user activity summary
    pub async fn get_user_activity(&self, limit: usize) -> Vec<(String, u64)> {
        self.hermes.get_top_users(limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hermes_creation() {
        let hermes = HermesAnalytics::new();
        assert_eq!(hermes.get_event_count().await, 0);
    }

    #[tokio::test]
    async fn test_record_event() {
        let hermes = HermesAnalytics::new();

        let event = Event::cache_hit("key:123".to_string());
        hermes.record_event(event).await;

        assert_eq!(hermes.get_event_count().await, 1);
    }

    #[tokio::test]
    async fn test_event_statistics() {
        let hermes = HermesAnalytics::new();

        for _ in 0..5 {
            let event = Event::cache_hit("key:123".to_string());
            hermes.record_event(event).await;
        }

        let stats = hermes.get_event_statistics().await;
        assert_eq!(stats.get("cache.hit"), Some(&5));
    }

    #[tokio::test]
    async fn test_user_activity_tracking() {
        let hermes = HermesAnalytics::new();

        let event1 = Event::file_uploaded("file:123".to_string(), "user:001".to_string());
        let event2 = Event::file_uploaded("file:456".to_string(), "user:001".to_string());
        let event3 = Event::file_uploaded("file:789".to_string(), "user:002".to_string());

        hermes.record_event(event1).await;
        hermes.record_event(event2).await;
        hermes.record_event(event3).await;

        let top_users = hermes.get_top_users(5).await;
        assert_eq!(top_users.len(), 2);
        assert_eq!(top_users[0].0, "user:001");
        assert_eq!(top_users[0].1, 2);
    }

    #[tokio::test]
    async fn test_analytics_report() {
        let hermes = HermesAnalytics::new();

        let event = Event::cache_hit("key:123".to_string());
        hermes.record_event(event).await;

        let report = hermes.generate_report(1).await;
        assert_eq!(report.total_events, 1);
        assert!(!report.event_stats.is_empty());
    }

    #[tokio::test]
    async fn test_statistics_collector() {
        let hermes = Arc::new(HermesAnalytics::new());
        let collector = StatisticsCollector::new(hermes);

        let event = Event::user_logged_in("user:001".to_string());
        collector.record(event).await;

        let summary = collector.get_summary().await;
        assert!(summary.contains_key("user.logged_in"));
    }
}
