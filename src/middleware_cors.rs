use actix_cors::Cors;
use actix_web::http;

pub fn configure_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://127.0.0.1:3000")
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_origin("tauri://localhost")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
        ])
        .supports_credentials()
        .max_age(3600)
}
