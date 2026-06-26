use actix_web::{web, App, HttpServer, middleware::Logger};
use crate::api;
use crate::auth;
use crate::config::OpenCodeConfig;
use crate::api::session::SessionStore;
use std::sync::Arc;

pub struct OpenCodeServer {
    pub config: OpenCodeConfig,
}

impl OpenCodeServer {
    pub fn new(config: OpenCodeConfig) -> Self {
        Self { config }
    }

    pub async fn start(self) -> std::io::Result<()> {
        let bind_addr = format!("{}:{}", self.config.host, self.config.port);
        let config = Arc::new(self.config);
        let session_store = crate::api::session::create_store();

        log::info!("OpenCode Server starting on {}", bind_addr);
        log::info!("Username: {}", config.username);
        log::info!("Password: {}", config.password);

        let basic_auth = auth::BasicAuth::new(
            config.username.clone(),
            config.password.clone(),
        );

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .wrap(auth::BasicAuthMiddleware {
                    auth: basic_auth.clone(),
                })
                .app_data(web::Data::new(session_store.clone()))
                .configure(api::router::configure)
        })
        .bind(&bind_addr)?
        .run()
        .await
    }
}
