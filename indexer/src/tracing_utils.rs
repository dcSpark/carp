use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_logging() {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        // sqlx logs every SQL query and how long it took which is very noisy
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("oura", tracing::Level::WARN)
        .with_target("carp", tracing::Level::TRACE)
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block
}
