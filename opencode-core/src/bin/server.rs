use opencode_core::{OpenCodeServer, config::OpenCodeConfig};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .init();

    let config = OpenCodeConfig::default();
    let mut server = OpenCodeServer::new(config);

    if let Ok(frontend_dir) = std::env::var("OPENCODE_FRONTEND_DIR") {
        server = server.with_frontend(std::path::PathBuf::from(frontend_dir));
    }

    server.start().await
}
