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
    use opentelemetry::global;
    use opentelemetry::sdk::Resource;
    use opentelemetry::sdk::trace::config;
    use opentelemetry_otlp::{SpanExporter, TonicExporter};
    use opentelemetry_semantic_conventions::resource::SEMCONV_RESOURCE_ATTRIBUTES;
    use tracing_opentelemetry::layer;
    use tracing_subscriber::{
        EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
    };

    let otlp_endpoint =
        std::env::var("OTLP_ENDPOINT").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| {
            tracing::info!("OTLP_ENDPOINT not set, falling back to console tracing");
            return String::new();
        });

    let registry = Registry::default();

    if !otlp_endpoint.is_empty() {
        let tracer_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                TonicExporter::builder()
                    .tonic_builder(tonic::transport::Channel::from_shared(otlp_endpoint).unwrap())
                    .build_span_exporter()
                    .expect("Failed to create OTLP exporter"),
            )
            .with_batch_config(
                config::BatchConfigBuilder::default()
                    .with_max_queue_size(2048)
                    .with_scheduled_delay_millis(5000)
                    .build(),
            )
            .with_resource(Resource::new(vec![(
                SEMCONV_RESOURCE_ATTRIBUTES.service_name,
                "app-http".into(),
            )]))
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to install OTLP tracer provider");

        let otel_layer = layer().with_tracer_provider(tracer_provider);

        let _ = registry
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(otel_layer)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .try_init();
    } else {
        // Console fallback
        let _ = registry
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .try_init();
    }
}

/// Initialize tracing for tests
///
/// Use this in test setup to get structured logs during test execution.
/// Compatible with OTLP setup.
#[cfg(test)]
pub fn init_test() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}
