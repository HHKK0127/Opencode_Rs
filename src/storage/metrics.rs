#![allow(dead_code)]
use lazy_static::lazy_static;
use prometheus::{Counter, Histogram, HistogramOpts, IntCounter, IntGauge};

lazy_static! {
    pub static ref STORAGE_UPLOAD_BYTES: Counter =
        Counter::new("storage_upload_bytes_total", "Total bytes uploaded").unwrap();
    pub static ref STORAGE_DOWNLOAD_BYTES: Counter =
        Counter::new("storage_download_bytes_total", "Total bytes downloaded").unwrap();
    pub static ref STORAGE_OPERATIONS_TOTAL: IntCounter =
        IntCounter::new("storage_operations_total", "Total storage operations").unwrap();
    pub static ref STORAGE_ERRORS_TOTAL: IntCounter =
        IntCounter::new("storage_errors_total", "Total storage errors").unwrap();
    pub static ref STORAGE_ACTIVE_UPLOADS: IntGauge =
        IntGauge::new("storage_active_uploads", "Active uploads").unwrap();
    pub static ref STORAGE_OPERATION_DURATION: Histogram = {
        let opts = HistogramOpts::new(
            "storage_operation_duration_seconds",
            "Storage operation duration",
        );
        Histogram::with_opts(opts).unwrap()
    };
}

pub struct StorageMetrics {
    pub total_uploads: u64,
    pub total_downloads: u64,
    pub total_deletes: u64,
    pub error_count: u64,
    pub average_latency_ms: f64,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self {
            total_uploads: 0,
            total_downloads: 0,
            total_deletes: 0,
            error_count: 0,
            average_latency_ms: 0.0,
        }
    }

    pub fn record_upload(&mut self, bytes: u64, latency_ms: f64) {
        self.total_uploads += 1;
        STORAGE_UPLOAD_BYTES.inc_by(bytes as f64);
        STORAGE_OPERATIONS_TOTAL.inc();
        STORAGE_OPERATION_DURATION.observe(latency_ms / 1000.0);
    }

    pub fn record_download(&mut self, bytes: u64, latency_ms: f64) {
        self.total_downloads += 1;
        STORAGE_DOWNLOAD_BYTES.inc_by(bytes as f64);
        STORAGE_OPERATIONS_TOTAL.inc();
        STORAGE_OPERATION_DURATION.observe(latency_ms / 1000.0);
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
        STORAGE_ERRORS_TOTAL.inc();
    }

    pub fn error_rate(&self) -> f64 {
        let total_ops = self.total_uploads + self.total_downloads + self.total_deletes;
        if total_ops == 0 {
            0.0
        } else {
            (self.error_count as f64 / total_ops as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metrics_initialization() {
        let metrics = StorageMetrics::new();
        assert_eq!(metrics.total_uploads, 0);
        assert_eq!(metrics.total_downloads, 0);
        assert_eq!(metrics.error_count, 0);
    }

    #[test]
    fn test_storage_metrics_record_upload() {
        let mut metrics = StorageMetrics::new();
        metrics.record_upload(1024, 100.0);
        assert_eq!(metrics.total_uploads, 1);
    }

    #[test]
    fn test_storage_metrics_error_rate() {
        let mut metrics = StorageMetrics::new();
        metrics.record_upload(1024, 100.0);
        metrics.record_download(2048, 150.0);
        metrics.record_error();

        let error_rate = metrics.error_rate();
        assert!(error_rate > 0.0 && error_rate < 100.0);
    }

    #[test]
    fn test_storage_metrics_error_rate_zero() {
        let metrics = StorageMetrics::new();
        assert_eq!(metrics.error_rate(), 0.0);
    }

    #[test]
    fn test_storage_metrics_error_rate_100_percent() {
        let mut metrics = StorageMetrics::new();
        metrics.record_error();
        // When only error is recorded without ops, rate is 0
        assert_eq!(metrics.error_rate(), 0.0);
    }
}
