use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub path: String,
    pub auto_init: bool,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub jwt_secret: String,
    pub token_expiry_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    pub max_file_size_mb: usize,
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Config {
    #[serde(default = "default_s3_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_s3_region")]
    pub region: String,
    #[serde(default = "default_s3_bucket")]
    pub bucket: String,
    #[serde(default = "default_s3_access_key")]
    pub access_key: String,
    #[serde(default = "default_s3_secret_key")]
    pub secret_key: String,
    #[serde(default = "default_use_path_style")]
    pub use_path_style: bool,
}

fn default_s3_endpoint() -> String {
    "http://localhost:9000".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn default_s3_bucket() -> String {
    "opencode-uploads".to_string()
}

fn default_s3_access_key() -> String {
    env::var("AWS_ACCESS_KEY")
        .or_else(|_| env::var("S3_ACCESS_KEY"))
        .unwrap_or_else(|_| "minioadmin".to_string())
}

fn default_s3_secret_key() -> String {
    env::var("AWS_SECRET_KEY")
        .or_else(|_| env::var("S3_SECRET_KEY"))
        .unwrap_or_else(|_| "minioadmin123".to_string())
}

fn default_use_path_style() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Presigned {
    #[serde(default = "default_put_expiry")]
    pub put_expiry_seconds: u64,
    #[serde(default = "default_get_expiry")]
    pub get_expiry_seconds: u64,
}

fn default_put_expiry() -> u64 {
    300
}

fn default_get_expiry() -> u64 {
    3600
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Multipart {
    #[serde(default = "default_chunk_size")]
    pub chunk_size_mb: usize,
    #[serde(default = "default_concurrent_parts")]
    pub max_concurrent_parts: usize,
}

fn default_chunk_size() -> usize {
    5
}

fn default_concurrent_parts() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Storage {
    #[serde(rename = "type")]
    #[serde(default = "default_storage_type")]
    pub storage_type: String,  // "s3" or "local"
    #[serde(default)]
    pub s3: S3Config,
    #[serde(default)]
    pub s3_presigned: S3Presigned,
    #[serde(default)]
    pub s3_multipart: S3Multipart,
}

fn default_storage_type() -> String {
    "s3".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub logging: Logging,
    pub auth: Auth,
    pub upload: Upload,
    #[serde(default)]
    pub storage: Storage,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", env)))
            .add_source(Environment::with_prefix("OPENCODE").try_parsing(true).separator("__"))
            .build()?;

        config.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: Server {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
            },
            database: Database {
                path: "./poc_test.db".to_string(),
                auto_init: true,
                max_connections: 5,
            },
            logging: Logging {
                level: "debug".to_string(),
                format: "compact".to_string(),
            },
            auth: Auth {
                jwt_secret: "dev_secret_key_change_in_production".to_string(),
                token_expiry_hours: 24,
            },
            upload: Upload {
                max_file_size_mb: 10,
                directory: "./uploads".to_string(),
            },
            storage: Storage {
                storage_type: "s3".to_string(),
                s3: S3Config {
                    endpoint: "http://localhost:9000".to_string(),
                    region: "us-east-1".to_string(),
                    bucket: "opencode-uploads".to_string(),
                    access_key: "minioadmin".to_string(),
                    secret_key: "minioadmin123".to_string(),
                    use_path_style: true,
                },
                s3_presigned: S3Presigned {
                    put_expiry_seconds: 300,
                    get_expiry_seconds: 3600,
                },
                s3_multipart: S3Multipart {
                    chunk_size_mb: 5,
                    max_concurrent_parts: 10,
                },
            },
        }
    }
}
