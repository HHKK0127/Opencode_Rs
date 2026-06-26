#[derive(Debug, Clone)]
pub struct OpenCodeConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub data_dir: String,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            username: std::env::var("OPENCODE_SERVER_USERNAME").unwrap_or_else(|_| "opencode".to_string()),
            password: std::env::var("OPENCODE_SERVER_PASSWORD").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            host: std::env::var("OPENCODE_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("OPENCODE_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: std::env::var("XDG_STATE_HOME")
                .unwrap_or_else(|_| {
                    dirs_data_dir()
                }),
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
