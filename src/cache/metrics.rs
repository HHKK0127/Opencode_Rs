use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, HistogramVec, IntGauge, Registry};

lazy_static! {
    pub static ref REDIS_CONNECTIONS_ACTIVE: IntGauge = IntGauge::new(
        "redis_connections_active",
        "Number of active Redis connections"
    )
    .expect("Failed to create redis_connections_active");
    pub static ref REDIS_COMMAND_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "redis_command_duration_seconds",
            "Redis command execution duration in seconds"
        ),
        &["command"],
    )
    .expect("Failed to create redis_command_duration_seconds");
    pub static ref REDIS_ERRORS_TOTAL: CounterVec = CounterVec::new(
        prometheus::Opts::new("redis_errors_total", "Total Redis errors"),
        &["error_type"],
    )
    .expect("Failed to create redis_errors_total");
    pub static ref REDIS_OPERATIONS_TOTAL: CounterVec = CounterVec::new(
        prometheus::Opts::new("redis_operations_total", "Total Redis operations"),
        &["operation"],
    )
    .expect("Failed to create redis_operations_total");
    pub static ref REDIS_CACHE_HITS_TOTAL: Counter =
        Counter::new("redis_cache_hits_total", "Total cache hits")
            .expect("Failed to create redis_cache_hits_total");
    pub static ref REDIS_CACHE_MISSES_TOTAL: Counter =
        Counter::new("redis_cache_misses_total", "Total cache misses")
            .expect("Failed to create redis_cache_misses_total");
}

pub fn register_redis_metrics(registry: &Registry) -> Result<(), Box<dyn std::error::Error>> {
    registry.register(Box::new(REDIS_CONNECTIONS_ACTIVE.clone()))?;
    registry.register(Box::new(REDIS_COMMAND_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(REDIS_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(REDIS_OPERATIONS_TOTAL.clone()))?;
    registry.register(Box::new(REDIS_CACHE_HITS_TOTAL.clone()))?;
    registry.register(Box::new(REDIS_CACHE_MISSES_TOTAL.clone()))?;
    Ok(())
}
