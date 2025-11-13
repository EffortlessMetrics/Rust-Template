/// Initialize tracing with env-based log filtering
///
/// Respects `RUST_LOG` environment variable for filtering.
/// Default level: INFO
///
/// Examples:
/// ```ignore
/// RUST_LOG=debug cargo run
/// RUST_LOG=app_http=trace,core=debug cargo run
/// ```
pub fn init() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Build the env filter
    // Default to INFO if RUST_LOG not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize tracing subscriber (idempotent - ignores error if already initialized)
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .try_init();
}

/// Initialize tracing for tests
///
/// Use this in test setup to get structured logs during test execution.
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}
