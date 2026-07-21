#![allow(dead_code)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

/// Signal for graceful shutdown
#[derive(Debug, Clone)]
pub enum ShutdownSignal {
    Sigterm,
    Sigint,
    Timeout,
}

/// Graceful shutdown manager
pub struct GracefulShutdown {
    shutdown_tx: broadcast::Sender<ShutdownSignal>,
    shutdown_rx: broadcast::Receiver<ShutdownSignal>,
    is_shutting_down: Arc<AtomicBool>,
    shutdown_timeout: Duration,
}

impl GracefulShutdown {
    /// Create new graceful shutdown handler
    pub fn new(timeout_secs: u64) -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(100);

        Self {
            shutdown_tx,
            shutdown_rx,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            shutdown_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Get a shutdown receiver for listening to shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownSignal> {
        self.shutdown_tx.subscribe()
    }

    /// Check if shutdown is in progress
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Relaxed)
    }

    /// Initiate graceful shutdown
    pub async fn shutdown(&self, signal: ShutdownSignal) {
        info!("Initiating graceful shutdown: {:?}", signal);

        // Mark as shutting down
        self.is_shutting_down.store(true, Ordering::Relaxed);

        // Broadcast shutdown signal
        if let Err(e) = self.shutdown_tx.send(signal) {
            warn!("Failed to broadcast shutdown signal: {}", e);
        }
    }

    /// Wait for shutdown signal (SIGTERM/SIGINT)
    pub async fn wait_for_signal(&self) {
        #[cfg(unix)]
        {
            use tokio::signal::unix::SignalKind;

            let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
                .expect("Failed to setup SIGTERM handler");

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    self.shutdown(ShutdownSignal::Sigint).await;
                }
                _ = sigterm.recv() => {
                    self.shutdown(ShutdownSignal::Sigterm).await;
                }
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl-C");
            self.shutdown(ShutdownSignal::Sigint).await;
        }
    }

    /// Perform graceful shutdown with timeout
    pub async fn perform_shutdown<F>(&self, shutdown_fn: F) -> Result<(), String>
    where
        F: std::future::Future<Output = Result<(), String>>,
    {
        info!(
            "Starting graceful shutdown sequence (timeout: {:?})",
            self.shutdown_timeout
        );

        match timeout(self.shutdown_timeout, shutdown_fn).await {
            Ok(Ok(())) => {
                info!("Graceful shutdown completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Graceful shutdown error: {}", e);
                Err(e)
            }
            Err(_) => {
                error!(
                    "Graceful shutdown timeout after {:?}",
                    self.shutdown_timeout
                );
                Err("Shutdown timeout".to_string())
            }
        }
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new(30) // 30 second default timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_creation() {
        let shutdown = GracefulShutdown::new(30);
        assert!(!shutdown.is_shutting_down());
    }

    #[tokio::test]
    async fn test_shutdown_flag() {
        let shutdown = GracefulShutdown::new(30);
        assert!(!shutdown.is_shutting_down());

        shutdown.shutdown(ShutdownSignal::Sigterm).await;
        assert!(shutdown.is_shutting_down());
    }

    #[tokio::test]
    async fn test_shutdown_broadcast() {
        let shutdown = GracefulShutdown::new(30);
        let mut rx = shutdown.subscribe();

        shutdown.shutdown(ShutdownSignal::Sigint).await;

        // Verify signal was broadcast
        match rx.recv().await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false, "Failed to receive shutdown signal"),
        }
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let shutdown = GracefulShutdown::new(1); // 1 second timeout

        let result = shutdown
            .perform_shutdown(async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok::<(), String>(())
            })
            .await;

        assert!(result.is_err(), "Should timeout");
    }
}
