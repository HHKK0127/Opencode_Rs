use std::sync::Arc;
use tokio::sync::RwLock;

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub heap_used_mb: f64,
    pub heap_total_mb: f64,
    pub cache_size_mb: f64,
    pub connection_count: usize,
    pub memory_pressure: f64, // 0.0 - 1.0
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            heap_used_mb: 0.0,
            heap_total_mb: 0.0,
            cache_size_mb: 0.0,
            connection_count: 0,
            memory_pressure: 0.0,
        }
    }

    pub fn update_pressure(&mut self) {
        if self.heap_total_mb > 0.0 {
            self.memory_pressure = self.heap_used_mb / self.heap_total_mb;
        }
    }

    pub fn is_high_pressure(&self) -> bool {
        self.memory_pressure > 0.8
    }

    pub fn is_critical(&self) -> bool {
        self.memory_pressure > 0.95
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory manager for monitoring and optimization
pub struct MemoryManager {
    stats: Arc<RwLock<MemoryStats>>,
    allocation_history: Arc<RwLock<Vec<(String, f64)>>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(MemoryStats::new())),
            allocation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Update memory statistics
    pub async fn update_stats(
        &self,
        heap_used: f64,
        heap_total: f64,
        cache_size: f64,
        connections: usize,
    ) {
        let mut stats = self.stats.write().await;
        stats.heap_used_mb = heap_used;
        stats.heap_total_mb = heap_total;
        stats.cache_size_mb = cache_size;
        stats.connection_count = connections;
        stats.update_pressure();
    }

    /// Get current memory stats
    pub async fn get_stats(&self) -> MemoryStats {
        self.stats.read().await.clone()
    }

    /// Record allocation
    pub async fn record_allocation(&self, component: String, size_mb: f64) {
        let mut history = self.allocation_history.write().await;
        history.push((component, size_mb));

        // Keep only last 1000 entries
        if history.len() > 1000 {
            let to_remove = history.len() - 1000;
            history.drain(0..to_remove);
        }
    }

    /// Get memory recommendations
    pub async fn get_recommendations(&self) -> Vec<String> {
        let stats = self.stats.read().await;
        let mut recommendations = Vec::new();

        if stats.is_critical() {
            recommendations.push("CRITICAL: Clear cache immediately".to_string());
            recommendations.push("CRITICAL: Close idle connections".to_string());
            recommendations.push("CRITICAL: Reduce memory-intensive operations".to_string());
        } else if stats.is_high_pressure() {
            recommendations.push("HIGH: Enable aggressive caching cleanup".to_string());
            recommendations.push("HIGH: Reduce cache retention period".to_string());
            recommendations.push("HIGH: Monitor memory growth".to_string());
        } else if stats.memory_pressure > 0.6 {
            recommendations.push("Optimize large queries".to_string());
            recommendations.push("Review cache hit rates".to_string());
            recommendations.push("Monitor connection pool size".to_string());
        }

        recommendations
    }

    /// Estimate memory usage
    pub async fn estimate_usage(&self) -> String {
        let stats = self.stats.read().await;
        let pressure_percent = (stats.memory_pressure * 100.0).round();

        format!(
            "Heap: {:.1}/{:.1}MB ({}%), Cache: {:.1}MB, Connections: {}",
            stats.heap_used_mb, stats.heap_total_mb, pressure_percent, stats.cache_size_mb, stats.connection_count
        )
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_creation() {
        let stats = MemoryStats::new();
        assert_eq!(stats.heap_used_mb, 0.0);
        assert_eq!(stats.memory_pressure, 0.0);
        assert!(!stats.is_high_pressure());
    }

    #[test]
    fn test_memory_pressure_calculation() {
        let mut stats = MemoryStats::new();
        stats.heap_used_mb = 850.0; // > 80%
        stats.heap_total_mb = 1000.0;
        stats.update_pressure();

        assert_eq!(stats.memory_pressure, 0.85);
        assert!(stats.is_high_pressure());
    }

    #[test]
    fn test_critical_pressure() {
        let mut stats = MemoryStats::new();
        stats.heap_used_mb = 960.0; // > 95%
        stats.heap_total_mb = 1000.0;
        stats.update_pressure();

        assert!(stats.is_critical());
        assert!(stats.is_high_pressure());
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let manager = MemoryManager::new();

        manager.update_stats(500.0, 1024.0, 100.0, 50).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.heap_used_mb, 500.0);
        assert_eq!(stats.connection_count, 50);
    }

    #[tokio::test]
    async fn test_memory_recommendations() {
        let manager = MemoryManager::new();

        manager.update_stats(950.0, 1000.0, 100.0, 100).await;

        let recommendations = manager.get_recommendations().await;
        assert!(!recommendations.is_empty());
    }
}
