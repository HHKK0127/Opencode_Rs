use opencode_core::{OpenCodeServer, config::OpenCodeConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = OpenCodeConfig::default();
    let server = OpenCodeServer::new(config);
    server.start().await
}
