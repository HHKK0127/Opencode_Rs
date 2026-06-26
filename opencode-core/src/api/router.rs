use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(super::health::global_health)
       .service(super::health::api_health)
       .service(super::config::global_config)
       .service(super::config::get_config)
       .service(super::config::config_providers)
       .service(super::session::list_sessions)
       .service(super::session::init_session)
       .service(super::session::get_session)
       .service(super::session::abort_session)
       .service(super::session::session_todo)
       .service(super::session::session_diff)
       .service(super::session::session_children)
       .service(super::provider::list_providers)
       .service(super::provider::update_provider)
       .service(super::provider::provider_oauth_authorize)
       .service(super::provider::provider_auth)
       .service(super::tools::tool_ids)
       .service(super::tools::list_tools)
       .service(super::find::find_file)
       .service(super::find::find_symbol);
}
