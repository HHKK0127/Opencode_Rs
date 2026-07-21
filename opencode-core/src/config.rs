#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub chrome_bin: Option<String>,
    pub headless: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub screenshot_dir: String,
    pub pdf_dir: String,
    pub max_concurrent_tabs: usize,
    pub navigation_timeout_secs: u64,
    pub screenshot_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_screenshot_size_mb: u64,
    pub chrome_args: Vec<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        let temp = std::env::temp_dir();
        Self {
            enabled: true,
            chrome_bin: std::env::var("CHROME_BIN").ok(),
            headless: true,
            window_width: 1280,
            window_height: 720,
            screenshot_dir: temp
                .join("opencode-screenshots")
                .to_string_lossy()
                .to_string(),
            pdf_dir: temp.join("opencode-pdf").to_string_lossy().to_string(),
            max_concurrent_tabs: 10,
            navigation_timeout_secs: 30,
            screenshot_timeout_secs: 10,
            idle_timeout_secs: 300,
            max_screenshot_size_mb: 10,
            chrome_args: vec![
                "--no-sandbox".into(),
                "--disable-dev-shm-usage".into(),
                "--disable-gpu".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum UiMode {
    #[default]
    App,
    Browser,
    None,
}

impl UiMode {
    pub fn from_env() -> Self {
        match std::env::var("OPENCODE_UI_MODE").as_deref() {
            Ok("browser") => UiMode::Browser,
            Ok("none") => UiMode::None,
            _ => UiMode::App,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenCodeConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub data_dir: String,
    pub browser: BrowserConfig,
    pub ui_mode: UiMode,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            username: std::env::var("OPENCODE_SERVER_USERNAME")
                .unwrap_or_else(|_| "opencode".to_string()),
            password: std::env::var("OPENCODE_SERVER_PASSWORD")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            host: std::env::var("OPENCODE_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("OPENCODE_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| dirs_data_dir()),
            browser: BrowserConfig::default(),
            ui_mode: UiMode::from_env(),
        }
    }
}

fn dirs_data_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|p| format!("{}\\opencode", p))
            .unwrap_or_else(|_| "./opencode-data".to_string())
    } else {
        std::env::var("HOME")
            .map(|p| format!("{}/.local/share/opencode", p))
            .unwrap_or_else(|_| "./opencode-data".to_string())
    }
}
