use actix_web::{get, web, HttpRequest, HttpResponse};
use std::path::{Component, Path, PathBuf};

pub fn configure_frontend(cfg: &mut web::ServiceConfig, frontend_dir: PathBuf) {
    cfg.app_data(web::Data::new(FrontendState { frontend_dir }))
        .service(serve_frontend);
}

#[derive(Clone)]
struct FrontendState {
    frontend_dir: PathBuf,
}

#[get("/{path:.*}")]
async fn serve_frontend(req: HttpRequest, state: web::Data<FrontendState>) -> HttpResponse {
    let path: PathBuf = req.match_info().query("path").parse().unwrap_or_default();
    let resolved = resolve_safe_path(&state.frontend_dir, &path);

    if resolved.is_file() {
        return serve_file(&resolved).await;
    }

    let index_html = state.frontend_dir.join("index.html");
    if index_html.is_file() {
        return serve_file(&index_html).await;
    }

    HttpResponse::NotFound().body("frontend not found")
}

fn resolve_safe_path(base: &Path, requested: &Path) -> PathBuf {
    let mut resolved = base.to_path_buf();
    for component in requested.components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::RootDir | Component::CurDir => {}
            _ => return base.to_path_buf(),
        }
    }

    if !resolved.starts_with(base) {
        return base.to_path_buf();
    }

    if resolved.is_dir() {
        resolved.push("index.html");
    }

    resolved
}

async fn serve_file(path: &Path) -> HttpResponse {
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let content_type = guess_content_type(path);
            HttpResponse::Ok().content_type(content_type).body(bytes)
        }
        Err(_) => HttpResponse::InternalServerError().body("failed to read file"),
    }
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("mjs") => "application/javascript",
        Some("css") => "text/css",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("json") => "application/json",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("aac") => "audio/aac",
        _ => "application/octet-stream",
    }
}
