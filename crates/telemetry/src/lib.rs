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
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(feature = "otlp")]
    {
        if let Ok(endpoint) = std::env::var("OTLP_ENDPOINT")
            && !endpoint.is_empty()
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
