use actix_web::web;
use std::path::PathBuf;

pub fn configure_api(cfg: &mut web::ServiceConfig) {
    // Health endpoints
    cfg.route("/global/health", web::get().to(super::health::global_health))
       .route("/api/health", web::get().to(super::health::api_health));

    // Config endpoints (dual routes)
    cfg.route("/global/config", web::get().to(super::config::global_config))
       .route("/config", web::get().to(super::config::get_config))
       .route("/api/config", web::get().to(super::config::get_config))
       .route("/config/providers", web::get().to(super::config::config_providers))
       .route("/api/config/providers", web::get().to(super::config::config_providers));

    // Session endpoints (V1, dual routes)
    cfg.route("/session", web::get().to(super::session::list_sessions))
       .route("/api/session", web::get().to(super::session::list_sessions))
       .route("/session/{id}/init", web::post().to(super::session::init_session))
       .route("/api/session/{id}/init", web::post().to(super::session::init_session))
       .route("/session/{id}", web::get().to(super::session::get_session))
       .route("/api/session/{id}", web::get().to(super::session::get_session))
       .route("/session/{id}/abort", web::post().to(super::session::abort_session))
       .route("/api/session/{id}/abort", web::post().to(super::session::abort_session))
       .route("/session/{id}/todo", web::get().to(super::session::session_todo))
       .route("/api/session/{id}/todo", web::get().to(super::session::session_todo))
       .route("/session/{id}/diff", web::get().to(super::session::session_diff))
       .route("/api/session/{id}/diff", web::get().to(super::session::session_diff))
       .route("/session/{id}/children", web::get().to(super::session::session_children))
       .route("/api/session/{id}/children", web::get().to(super::session::session_children));

    // Session endpoints (V2)
    cfg.route("/api/session", web::post().to(super::session::create_session_v2))
       .route("/v2/session", web::post().to(super::session::create_session_v2))
       .route("/api/session", web::get().to(super::session::list_sessions_v2))
       .route("/v2/session", web::get().to(super::session::list_sessions_v2))
       .route("/api/session/{id}", web::get().to(super::session::get_session_v2))
       .route("/v2/session/{id}", web::get().to(super::session::get_session_v2))
       .route("/api/session/{id}", web::delete().to(super::session::delete_session_v2))
       .route("/v2/session/{id}", web::delete().to(super::session::delete_session_v2))
       .route("/api/session/{id}/message", web::get().to(super::session::get_session_messages))
       .route("/v2/session/{id}/message", web::get().to(super::session::get_session_messages));

    // Prompt endpoints (V2)
    cfg.route("/api/session/{id}/prompt", web::post().to(super::prompt::prompt_v2))
       .route("/v2/session/{id}/prompt", web::post().to(super::prompt::prompt_v2));

    // Prompt endpoints (V1 async)
    cfg.route("/session/{id}/prompt_async", web::post().to(super::prompt::prompt_async_v1));

    // Event subscription (SSE)
    cfg.route("/api/event", web::get().to(super::events::subscribe_events))
       .route("/v2/event", web::get().to(super::events::subscribe_events));

    // Question endpoints (V2)
    cfg.route("/api/session/{id}/question", web::get().to(super::question::list_questions))
       .route("/v2/session/{id}/question", web::get().to(super::question::list_questions))
       .route("/api/session/{id}/question/{request_id}/reply", web::post().to(super::question::reply_question))
       .route("/v2/session/{id}/question/{request_id}/reply", web::post().to(super::question::reply_question))
       .route("/api/session/{id}/question/{request_id}/reject", web::post().to(super::question::reject_question))
       .route("/v2/session/{id}/question/{request_id}/reject", web::post().to(super::question::reject_question));

    // Permission endpoints (V2)
    cfg.route("/api/session/{id}/permission", web::get().to(super::permission::list_permissions))
       .route("/v2/session/{id}/permission", web::get().to(super::permission::list_permissions))
       .route("/api/session/{id}/permission/{request_id}/reply", web::post().to(super::permission::reply_permission))
       .route("/v2/session/{id}/permission/{request_id}/reply", web::post().to(super::permission::reply_permission));

    // Provider endpoints (dual routes)
    cfg.route("/provider", web::get().to(super::provider::list_providers))
       .route("/api/provider", web::get().to(super::provider::list_providers))
       .route("/provider/{id}", web::put().to(super::provider::update_provider))
       .route("/api/provider/{id}", web::put().to(super::provider::update_provider))
       .route("/provider/{id}/oauth/authorize", web::get().to(super::provider::provider_oauth_authorize))
       .route("/api/provider/{id}/oauth/authorize", web::get().to(super::provider::provider_oauth_authorize))
       .route("/provider/auth", web::get().to(super::provider::provider_auth))
       .route("/api/provider/auth", web::get().to(super::provider::provider_auth));

    // Tool endpoints (experimental only)
    cfg.route("/experimental/tool/ids", web::get().to(super::tools::tool_ids))
       .route("/experimental/tool", web::get().to(super::tools::list_tools));

    // Find endpoints (dual routes)
    cfg.route("/find/file", web::get().to(super::find::find_file))
        .route("/api/find/file", web::get().to(super::find::find_file))
        .route("/find/symbol", web::get().to(super::find::find_symbol))
        .route("/api/find/symbol", web::get().to(super::find::find_symbol));

    // Browser control endpoints
    cfg.service(
        web::scope("/api/browser")
            .route("/navigate", web::post().to(super::browser::navigate))
            .route("/find_tab", web::post().to(super::browser::find_tab))
            .route("/snapshot", web::get().to(super::browser::snapshot))
            .route("/click", web::post().to(super::browser::click))
            .route("/fill", web::post().to(super::browser::fill))
            .route("/evaluate", web::post().to(super::browser::evaluate))
            .route("/screenshot", web::post().to(super::browser::screenshot))
            .route("/save_as_pdf", web::post().to(super::browser::save_as_pdf))
            .route("/list_tabs", web::get().to(super::browser::list_tabs))
            .route("/close_tab", web::post().to(super::browser::close_tab))
            .route("/close_session", web::post().to(super::browser::close_session))
            .route("/status", web::get().to(super::browser::status)),
    );
}

pub fn configure(cfg: &mut web::ServiceConfig, frontend_dir: Option<PathBuf>) {
    configure_api(cfg);
    if let Some(dir) = frontend_dir {
        super::static_files::configure_frontend(cfg, dir);
    }
}
