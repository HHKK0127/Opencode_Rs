use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;
use once_cell::sync::Lazy;

/// JWT_SECRET - 環境変数から取得（起動時に必須チェック）
/// 生成方法: export JWT_SECRET=$(openssl rand -base64 32)
pub static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set. Generate with: openssl rand -base64 32")
});

/// JWT有効期限（時間）- デフォルト24時間
pub static JWT_EXPIRATION_HOURS: Lazy<u32> = Lazy::new(|| {
    env::var("JWT_EXPIRATION_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24)
});

/// Argon2 メモリコスト（KB）- OWASP推奨: 64MB (65536 KB)
pub static ARGON2_MEMORY_COST: Lazy<u32> = Lazy::new(|| {
    env::var("ARGON2_MEMORY_COST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(65536)
});

/// Argon2 反復回数 - OWASP推奨: 3
pub static ARGON2_TIME_COST: Lazy<u32> = Lazy::new(|| {
    env::var("ARGON2_TIME_COST")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3)
});

/// Argon2 並列度 - OWASP推奨: 4
pub static ARGON2_PARALLELISM: Lazy<u32> = Lazy::new(|| {
    env::var("ARGON2_PARALLELISM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
});

/// Redis必須フラグ
pub static REDIS_REQUIRED: Lazy<bool> = Lazy::new(|| {
    env::var("REDIS_REQUIRED")
        .map(|s| s == "true")
        .unwrap_or(false)
});

/// DATABASE_URL - PostgreSQL / SQLite フォールバック
/// PostgreSQL: postgresql://user:pass@host:port/dbname
/// SQLite: sqlite://./poc_test.db (デフォルト)
pub static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, using SQLite fallback: sqlite://./poc_test.db");
        "sqlite://./poc_test.db".to_string()
    })
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub auto_init: bool,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub jwt_secret: String,  // 実行時には使用せず、JWT_SECRETを直接使用
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
    pub storage_type: String,
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
        // .envファイル読み込み（開発環境）
        if env::var("RUST_ENV").unwrap_or_default() == "development" {
            let _ = dotenv::dotenv();
        }

        // JWT_SECRETが設定されているか検証
        let _ = JWT_SECRET.as_str(); // ここで未設定ならpanic

        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(Environment::with_prefix("OPENCODE").try_parsing(true).separator("__"))
            .build()?;

        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_expiration_default() {
        std::env::set_var("JWT_SECRET", "test_secret_for_testing_only");
        assert_eq!(*JWT_EXPIRATION_HOURS, 24);
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
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/opencode".to_string()),
                auto_init: true,
                max_connections: 50,
                min_connections: 5,
                acquire_timeout_secs: 5,
            },
            logging: Logging {
                level: "info".to_string(),
                format: "compact".to_string(),
            },
            auth: Auth {
                jwt_secret: "MUST_BE_SET_FROM_ENV".to_string(),
                token_expiry_hours: *JWT_EXPIRATION_HOURS,
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
