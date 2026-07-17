// Public library interface for testing and integration
pub mod app_state;
pub mod cache;  // Wave 4: Redis caching layer
pub mod config;
pub mod error;
pub mod models;
pub mod auth_middleware;
pub mod middleware;
pub mod middleware_cors;
pub mod middleware_logging;
pub mod middleware_rate_limit;
pub mod api;
pub mod db;
pub mod storage;  // Wave 3: S3/MinIO integration
pub mod validation;  // Wave 3 Day 7: Input validation & security
pub mod notifications;  // Wave 4.5: WebSocket + Hermes analytics
pub mod optimization;   // Wave 5 Day 20: Performance optimization
pub mod production;     // Wave 5 Day 21: Production configuration
pub mod graceful;       // Wave 5 Day 24: Graceful shutdown & connection management
pub mod health_check_integration; // Wave 5 Day 24: Health check + Graceful shutdown integration
pub mod structured_logging;       // Wave 5 Day 24: Structured logging with tracing
pub mod event_persistence;        // Wave 5 Day 24: Event persistence with Redis
pub mod startup;                  // Wave 6: Startup validation & initialization
pub mod tui;                      // Wave 7: TUI compositor framework
