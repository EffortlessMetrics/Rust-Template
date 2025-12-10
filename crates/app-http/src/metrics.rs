/// Prometheus metrics for HTTP endpoints
///
/// This module provides:
/// - Global metrics registry
/// - HTTP request counter with labels (method, path, status)
/// - HTTP request duration histogram with labels (method, path, status_code)
/// - Middleware to automatically record requests and latency
/// - `/metrics` endpoint handler
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use std::time::Instant;

/// Global Prometheus registry
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// HTTP requests total counter
///
/// Labels:
/// - `method`: HTTP method (GET, POST, etc.)
/// - `path`: Request path
/// - `status`: HTTP status code
static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let opts = Opts::new("http_requests_total", "Total number of HTTP requests processed");
    let counter = IntCounterVec::new(opts, &["method", "path", "status"])
        .expect("Failed to create HTTP_REQUESTS_TOTAL metric");

    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("Failed to register HTTP_REQUESTS_TOTAL metric");

    counter
});

/// HTTP request duration histogram
///
/// Labels:
/// - `method`: HTTP method (GET, POST, etc.)
/// - `path`: Request path
/// - `status_code`: HTTP status code
///
/// Buckets: Standard Prometheus latency buckets (5ms to 10s)
static HTTP_REQUEST_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts =
        HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);

    let histogram = HistogramVec::new(opts, &["method", "path", "status_code"])
        .expect("Failed to create HTTP_REQUEST_DURATION_SECONDS metric");

    REGISTRY
        .register(Box::new(histogram.clone()))
        .expect("Failed to register HTTP_REQUEST_DURATION_SECONDS metric");

    histogram
});

/// Metrics endpoint handler
///
/// Returns Prometheus metrics in text format
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            let metrics_text = String::from_utf8(buffer).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to convert metrics to UTF-8");
                String::from("# Error: failed to encode metrics")
            });
            (StatusCode::OK, metrics_text).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response()
        }
    }
}

/// Middleware to record HTTP metrics
///
/// Records each request with method, path, and status labels
/// Tracks both request count and duration histogram
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status().as_u16().to_string();
    let elapsed = start.elapsed();

    // Record metrics
    HTTP_REQUESTS_TOTAL.with_label_values(&[&method, &path, &status]).inc();
    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &path, &status])
        .observe(elapsed.as_secs_f64());

    tracing::debug!(
        method = %method,
        path = %path,
        status = %status,
        elapsed_ms = elapsed.as_millis(),
        "HTTP request processed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counter_increments() {
        // Increment counter
        HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/test", "200"]).inc();

        // Verify it was recorded
        let metric_families = REGISTRY.gather();
        let http_requests = metric_families.iter().find(|mf| mf.name() == "http_requests_total");

        assert!(http_requests.is_some(), "http_requests_total metric should be registered");
    }

    #[tokio::test]
    async fn test_metrics_handler_returns_text() {
        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
