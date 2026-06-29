use actix_web::{web, App, HttpServer, middleware::Logger};
use crate::api;
use crate::auth;
use crate::config::OpenCodeConfig;
use std::path::PathBuf;
use std::sync::Arc;

pub struct OpenCodeServer {
    pub config: OpenCodeConfig,
    pub frontend_dir: Option<PathBuf>,
}

impl OpenCodeServer {
    pub fn new(config: OpenCodeConfig) -> Self {
        Self {
            config,
            frontend_dir: None,
        }
    }

    pub fn with_frontend(mut self, dir: PathBuf) -> Self {
        self.frontend_dir = Some(dir);
        self
    }

    pub async fn start(self) -> std::io::Result<()> {
        let bind_addr = format!("{}:{}", self.config.host, self.config.port);
        let config = Arc::new(self.config);
        let session_store = api::session::create_store();
        let event_bus = api::events::create_event_bus();
        let question_store = api::question::create_question_store();
        let permission_store = api::permission::create_permission_store();
        let frontend_dir = self.frontend_dir.clone();

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
                .app_data(web::Data::new(event_bus.clone()))
                .app_data(web::Data::new(question_store.clone()))
                .app_data(web::Data::new(permission_store.clone()))
                .configure(|cfg| api::router::configure(cfg, frontend_dir.clone()))
        })
        .bind(&bind_addr)?
        .run()
        .await
    }
}
