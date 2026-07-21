use opencode_core::{config::OpenCodeConfig, OpenCodeServer};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .init();

    let config = OpenCodeConfig::default();
    let mut server = OpenCodeServer::new(config);

    let frontend_dir = std::env::var("OPENCODE_FRONTEND_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // Default: look for opencode-desktop/dist relative to the project root
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("opencode-desktop")
                .join("dist")
        });
    server = server.with_frontend(frontend_dir);

    server.start().await
}
