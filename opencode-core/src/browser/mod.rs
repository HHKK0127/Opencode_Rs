pub mod accessibility;
pub mod error;
pub mod session;
pub mod tools;
pub mod types;

use chromiumoxide::browser::{Browser, BrowserConfig as ChromeConfig};
use futures::StreamExt;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub use error::BrowserError;

pub struct BrowserManager {
    browser: Option<Browser>,
    handler_task: Option<tokio::task::JoinHandle<()>>,
    sessions: session::SessionStore,
    config: crate::config::BrowserConfig,
}

impl BrowserManager {
    pub async fn launch(config: &crate::config::BrowserConfig) -> Result<Self, BrowserError> {
        let chrome_path = config
            .chrome_bin
            .clone()
            .or_else(|| {
                which::which("chrome")
                    .ok()
                    .or_else(|| which::which("google-chrome").ok())
                    .or_else(|| which::which("chromium").ok())
                    .or_else(|| which::which("chromium-browser").ok())
                    .map(|p| p.to_string_lossy().to_string())
            })
            .ok_or(BrowserError::ChromeNotFound)?;

        tracing::info!("Using Chrome binary: {}", chrome_path);

        let mut builder = ChromeConfig::builder()
            .chrome_executable(&chrome_path)
            .window_size(config.window_width, config.window_height)
            .no_sandbox()
            .arg("--disable-dev-shm-usage");

        if config.headless {
            builder = builder.arg("--headless");
        }

        for arg in &config.chrome_args {
            builder = builder.arg(arg.as_str());
        }

        let chrome_config = builder
            .build()
            .map_err(|e| BrowserError::ChromeLaunchFailed(e.to_string()))?;

        let (browser, mut handler) = Browser::launch(chrome_config)
            .await
            .map_err(|e| BrowserError::ChromeLaunchFailed(e.to_string()))?;

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    tracing::error!("Browser handler error: {}", e);
                    break;
                }
            }
        });

        tokio::time::timeout(std::time::Duration::from_secs(30), browser.version())
            .await
            .map_err(|_| BrowserError::ConnectionFailed("Health check timeout".into()))?
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;

        std::fs::create_dir_all(&config.screenshot_dir)?;
        std::fs::create_dir_all(&config.pdf_dir)?;

        tracing::info!("Browser manager launched successfully");

        Ok(Self {
            browser: Some(browser),
            handler_task: Some(handler_task),
            sessions: RwLock::new(HashMap::new()),
            config: config.clone(),
        })
    }

    pub async fn shutdown(mut self) -> Result<(), BrowserError> {
        {
            let mut sessions = self.sessions.write().await;
            sessions.clear();
        }

        if let Some(task) = self.handler_task.take() {
            task.abort();
        }

        if let Some(mut browser) = self.browser.take() {
            browser
                .close()
                .await
                .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;
        }

        tracing::info!("Browser manager shut down");
        Ok(())
    }

    pub async fn status(&self) -> serde_json::Value {
        let alive = if let Some(ref browser) = self.browser {
            browser.version().await.is_ok()
        } else {
            false
        };
        serde_json::json!({
            "running": alive,
            "session_count": self.sessions.read().await.len(),
        })
    }

    pub fn config(&self) -> &crate::config::BrowserConfig {
        &self.config
    }
}

impl Drop for BrowserManager {
    fn drop(&mut self) {
        if self.browser.is_some() {
            tracing::warn!(
                "BrowserManager dropped without calling shutdown(). Chrome process may be orphaned."
            );
        }
    }
}
