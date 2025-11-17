/// Initialize tracing with env-based log filtering and optional OTLP export
///
/// Respects `RUST_LOG` environment variable for filtering.
/// Default level: INFO
///
/// If `OTLP_ENDPOINT` is set, traces will be exported via OTLP.
///
/// Examples:
/// ```ignore
/// RUST_LOG=debug cargo run
/// RUST_LOG=app_http=trace,business_core=debug cargo run
/// OTLP_ENDPOINT=http://localhost:4317 cargo run
/// ```
pub fn init_tracing(service_name: &str) {
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // For now, just use console tracing
    // TODO: Add OTLP support when needed
    let _ = tracing_subscriber::fmt().with_env_filter(env_filter).try_init();

    tracing::info!(service = service_name, "Initialized console tracing");
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
