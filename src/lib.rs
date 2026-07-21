#![allow(
    dead_code,
    unused_variables,
    unused_assignments,
    unused_imports,
    unreachable_patterns,
    async_fn_in_trait,
    missing_docs,
    clippy::all
)]

// Public library interface for testing and integration
pub mod api;
pub mod app_state;
pub mod auth_middleware;
pub mod cache; // Wave 4: Redis caching layer
pub mod config;
pub mod db;
pub mod error;
pub mod event_persistence; // Wave 5 Day 24: Event persistence with Redis
pub mod graceful; // Wave 5 Day 24: Graceful shutdown & connection management
pub mod health_check_integration; // Wave 5 Day 24: Health check + Graceful shutdown integration
pub mod middleware;
pub mod middleware_cors;
pub mod middleware_logging;
pub mod middleware_rate_limit;
pub mod models;
pub mod notifications; // Wave 4.5: WebSocket + Hermes analytics
pub mod optimization; // Wave 5 Day 20: Performance optimization
pub mod production; // Wave 5 Day 21: Production configuration
pub mod startup; // Wave 6: Startup validation & initialization
pub mod storage; // Wave 3: S3/MinIO integration
pub mod structured_logging; // Wave 5 Day 24: Structured logging with tracing
pub mod tui;
pub mod validation; // Wave 3 Day 7: Input validation & security // Wave 7: TUI compositor framework
