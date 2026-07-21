use crate::api;
use crate::auth;
use crate::browser::BrowserManager;
use crate::config::{OpenCodeConfig, UiMode};
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

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
        let ui_mode = config.ui_mode.clone();
        let url = format!("http://{}:{}", config.host, config.port);
        let session_store = api::session::create_store();
        let event_bus = api::events::create_event_bus();
        let question_store = api::question::create_question_store();
        let permission_store = api::permission::create_permission_store();
        let frontend_dir = self.frontend_dir.clone();

        log::info!("OpenCode Server starting on {}", bind_addr);
        log::info!("Username: {}", config.username);
        log::info!("Password: {}", config.password);

        let health_state = web::Data::new(api::health::HealthState {
            start_time: Instant::now(),
            session_store: session_store.clone(),
            event_bus: event_bus.clone(),
        });

        let browser_manager = if config.browser.enabled {
            match BrowserManager::launch(&config.browser).await {
                Ok(mgr) => {
                    log::info!("Browser manager launched successfully");
                    Some(web::Data::new(mgr))
                }
                Err(e) => {
                    log::warn!(
                        "Browser manager launch failed: {}. Browser features disabled.",
                        e
                    );
                    None
                }
            }
        } else {
            log::info!("Browser features disabled by config");
            None
        };

        let basic_auth = auth::BasicAuth::new(config.username.clone(), config.password.clone());

        let server = HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Logger::default())
                .wrap(auth::BasicAuthMiddleware {
                    auth: basic_auth.clone(),
                })
                .app_data(health_state.clone())
                .app_data(web::Data::new(session_store.clone()))
                .app_data(web::Data::new(event_bus.clone()))
                .app_data(web::Data::new(question_store.clone()))
                .app_data(web::Data::new(permission_store.clone()))
                .configure(|cfg| api::router::configure(cfg, frontend_dir.clone()));

            if let Some(ref mgr) = browser_manager {
                app = app.app_data(mgr.clone());
            }

            app
        })
        .bind(&bind_addr)?
        .run();

        match ui_mode {
            UiMode::App => launch_edge_app(&url),
            UiMode::Browser => open_browser(&url),
            UiMode::None => log::info!("UI mode: none. Access the server at {}", url),
        }

        server.await
    }
}

fn launch_edge_app(url: &str) {
    launch_via_shellexecute(url);
}

#[cfg(not(target_os = "windows"))]
fn launch_via_shellexecute(_url: &str) {
    // Non-Windows fallback: use xdg-open or open
    let _ = open_browser(_url);
}

#[cfg(target_os = "windows")]
fn launch_via_shellexecute(url: &str) {
    let edge_paths = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe\0",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe\0",
    ];
    let edge = edge_paths.iter().find(|p| {
        let trimmed = p.trim_end_matches('\0');
        std::path::Path::new(trimmed).exists()
    });

    let exe = match edge {
        Some(p) => &p[..p.len() - 2],
        None => "msedge.exe",
    };

    let args = format!("--app={}", url);
    let args_utf16: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();
    let exe_utf16: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(),
            std::ptr::null(),
            exe_utf16.as_ptr(),
            args_utf16.as_ptr(),
            std::ptr::null_mut(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWDEFAULT,
        )
    };

    if result as usize > 32 {
        log::info!("Edge app window launched: {}", url);
    } else {
        log::warn!(
            "ShellExecuteW failed (code {}), falling back to cmd",
            result as usize
        );
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", exe, &args])
            .spawn();
    }
}

fn open_browser(url: &str) {
    let result = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };

    match result {
        Ok(_) => log::info!("Browser opened: {}", url),
        Err(e) => log::warn!("Failed to open browser: {}", e),
    }
}
