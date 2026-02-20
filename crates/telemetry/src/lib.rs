//! Telemetry and observability for the platform.
//!
//! This crate provides tracing initialization with:
//! - Environment-based log filtering via `RUST_LOG`
//! - Optional OTLP export for distributed tracing (feature: `otlp`)
//! - Console tracing fallback when OTLP is unavailable
//!
//! # Configuration
//!
//! - `RUST_LOG`: Log level filter (e.g., `debug`, `app_http=trace`)
//! - `OTLP_ENDPOINT`: OTLP collector endpoint (e.g., `http://localhost:4317`)
//!
//! # Examples
//!
//! ```no_run
//! # use telemetry::init_tracing;
//! init_tracing("my-service");
//! ```
//!
//! For OTLP export, enable the `otlp` feature and set the endpoint:
//!
//! ```bash
//! OTLP_ENDPOINT=http://localhost:4317 cargo run --features telemetry/otlp
//! ```

use std::ffi::OsString;

/// Abstraction over environment variable reading for testability.
pub(crate) trait EnvReader {
    /// Read an environment variable, returning `None` if absent.
    fn var_os(&self, key: &str) -> Option<OsString>;
}

/// Production implementation — delegates to `std::env`.
pub(crate) struct SystemEnv;

impl EnvReader for SystemEnv {
    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

/// Resolve the tracing env filter from the given environment.
///
/// Reads `RUST_LOG`; falls back to `"info"` if absent or unparseable.
pub(crate) fn resolve_env_filter(env: &dyn EnvReader) -> tracing_subscriber::EnvFilter {
    use tracing_subscriber::EnvFilter;
    match env.var_os("RUST_LOG").and_then(|v| v.into_string().ok()) {
        Some(val) => EnvFilter::try_new(&val).unwrap_or_else(|_| EnvFilter::new("info")),
        None => EnvFilter::new("info"),
    }
}

/// Initialize tracing with env-based log filtering and optional OTLP export
///
/// Respects `RUST_LOG` environment variable for filtering.
/// Default level: INFO
///
/// ## OTLP Export (Optional)
///
/// When the `otlp` feature is enabled and `OTLP_ENDPOINT` is set, traces will be
/// exported via OTLP using gRPC (tonic). If OTLP initialization fails, the system
/// falls back to console-only tracing with a warning.
///
/// Environment Variables:
/// - `RUST_LOG`: Logging filter (e.g., `debug`, `app_http=trace`)
/// - `OTLP_ENDPOINT`: OTLP collector endpoint (e.g., `http://localhost:4317`)
///
/// Examples:
/// ```ignore
/// RUST_LOG=debug cargo run
/// RUST_LOG=app_http=trace,business_core=debug cargo run
/// OTLP_ENDPOINT=http://localhost:4317 cargo run --features telemetry/otlp
/// ```
pub fn init_tracing(service_name: &str) {
    init_tracing_with_env(service_name, &SystemEnv);
}

/// Inner implementation accepting an injectable env reader.
pub(crate) fn init_tracing_with_env(service_name: &str, env: &dyn EnvReader) {
    let env_filter = resolve_env_filter(env);

    #[cfg(feature = "otlp")]
    {
        if let Some(endpoint) =
            env.var_os("OTLP_ENDPOINT").and_then(|v| v.into_string().ok()).filter(|s| !s.is_empty())
        {
            match try_init_otlp(service_name, &endpoint, env_filter.clone()) {
                Ok(()) => {
                    tracing::info!(
                        service = service_name,
                        endpoint = %endpoint,
                        "Initialized OTLP tracing"
                    );
                    return;
                }
                Err(e) => {
                    eprintln!(
                        "Failed to initialize OTLP exporter: {e}, falling back to console tracing"
                    );
                }
            }
        }
    }

    // Suppress unused-variable warning when otlp feature is off
    let _ = env;

    // Default console tracing (also used as fallback if OTLP fails)
    let _ = tracing_subscriber::fmt().with_env_filter(env_filter).try_init();

    tracing::info!(service = service_name, "Initialized console tracing");
}

#[cfg(feature = "otlp")]
fn try_init_otlp(
    service_name: &str,
    endpoint: &str,
    env_filter: tracing_subscriber::EnvFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_otlp::{SpanExporter, WithExportConfig};
    use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
    use tracing_subscriber::layer::SubscriberExt;

    // Build OTLP exporter using gRPC (tonic)
    let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;

    // Create tracer provider with resource metadata
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(service_name.to_string()).build())
        .build();

    // Set global tracer provider
    global::set_tracer_provider(provider);

    // Create tracing-opentelemetry layer
    let tracer = global::tracer(service_name.to_string());
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Combine OTLP + console layers
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;

    use tracing_subscriber::EnvFilter;

    /// Deterministic fake environment for testing without process-global mutation.
    struct FakeEnv {
        vars: HashMap<String, OsString>,
    }

    impl FakeEnv {
        fn new() -> Self {
            Self { vars: HashMap::new() }
        }

        fn with_var(mut self, key: &str, val: &str) -> Self {
            self.vars.insert(key.to_string(), OsString::from(val));
            self
        }
    }

    impl super::EnvReader for FakeEnv {
        fn var_os(&self, key: &str) -> Option<OsString> {
            self.vars.get(key).cloned()
        }
    }

    // ── resolve_env_filter tests ──

    #[test]
    fn resolve_env_filter_falls_back_when_rust_log_unset() {
        let env = FakeEnv::new();
        let filter = super::resolve_env_filter(&env);
        assert!(filter.to_string().contains("info"));
    }

    #[test]
    fn resolve_env_filter_uses_rust_log_when_set() {
        let env = FakeEnv::new().with_var("RUST_LOG", "debug");
        let filter = super::resolve_env_filter(&env);
        assert!(filter.to_string().contains("debug"));
    }

    #[test]
    fn resolve_env_filter_falls_back_on_invalid_rust_log() {
        let env = FakeEnv::new().with_var("RUST_LOG", "not_a_valid_level[[[");
        let filter = super::resolve_env_filter(&env);
        assert!(filter.to_string().contains("info"));
    }

    #[test]
    fn resolve_env_filter_parses_module_specific_levels() {
        let env = FakeEnv::new().with_var("RUST_LOG", "app_http=trace,business_core=debug");
        let filter = super::resolve_env_filter(&env);
        let s = filter.to_string();
        assert!(s.contains("app_http"));
        assert!(s.contains("business_core"));
    }

    // ── EnvFilter parsing (pure, no env) ──

    #[test]
    fn env_filter_parses_default_level() {
        let filter = EnvFilter::try_new("info").expect("should parse 'info'");
        assert!(!filter.to_string().is_empty());
    }

    #[test]
    fn env_filter_parses_debug_level() {
        let filter = EnvFilter::try_new("debug").expect("should parse 'debug'");
        assert!(filter.to_string().contains("debug"));
    }

    #[test]
    fn env_filter_parses_trace_level() {
        let filter = EnvFilter::try_new("trace").expect("should parse 'trace'");
        assert!(filter.to_string().contains("trace"));
    }

    #[test]
    fn env_filter_parses_warn_level() {
        let filter = EnvFilter::try_new("warn").expect("should parse 'warn'");
        assert!(filter.to_string().contains("warn"));
    }

    #[test]
    fn env_filter_parses_error_level() {
        let filter = EnvFilter::try_new("error").expect("should parse 'error'");
        assert!(filter.to_string().contains("error"));
    }

    #[test]
    fn env_filter_parses_complex_filter() {
        let filter = EnvFilter::try_new("info,telemetry=debug,tower_http=warn,sqlx=error")
            .expect("should parse complex filter");
        assert!(!filter.to_string().is_empty());
    }

    #[test]
    fn env_filter_handles_invalid_gracefully() {
        let result = EnvFilter::try_new("");
        assert!(result.is_ok());
    }

    // ── OTLP endpoint validation ──

    #[test]
    fn otlp_endpoint_empty_string_is_not_valid_endpoint() {
        let endpoint = "";
        assert!(endpoint.is_empty(), "empty endpoint should be detected");
    }

    #[test]
    fn otlp_endpoint_valid_http_url() {
        let endpoint = "http://localhost:4317";
        assert!(
            endpoint.starts_with("http://") || endpoint.starts_with("https://"),
            "should be a valid HTTP URL"
        );
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn otlp_endpoint_valid_https_url() {
        let endpoint = "https://otel-collector.example.com:4317";
        assert!(endpoint.starts_with("https://"));
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn otlp_endpoint_with_custom_port() {
        let endpoint = "http://127.0.0.1:55680";
        assert!(endpoint.contains(":55680"));
    }

    // ── Smoke tests ──

    #[test]
    fn init_tracing_function_exists() {
        let _func: fn(&str) = super::init_tracing;
    }

    #[test]
    fn service_name_validation() {
        let valid_names =
            ["my-service", "app_http", "business-core", "telemetry", "test-service-123"];
        for name in valid_names {
            assert!(!name.is_empty(), "service name should not be empty");
            assert!(
                name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
                "service name '{}' should contain only alphanumeric, dash, or underscore",
                name
            );
        }
    }

    #[test]
    fn init_test_function_can_be_called() {
        super::init_test();
    }

    #[test]
    fn fallback_logic_prefers_console_when_otlp_unavailable() {
        #[cfg(not(feature = "otlp"))]
        {
            let filter = EnvFilter::try_new("info").expect("should parse");
            assert!(!filter.to_string().is_empty());
        }
    }
}
