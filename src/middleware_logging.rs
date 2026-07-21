use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .compact()
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    log::info!("Logging initialized");
}
