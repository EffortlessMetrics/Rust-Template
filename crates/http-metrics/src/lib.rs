//! Shared Prometheus metrics for HTTP services.
//!
//! This crate provides:
//! - A `/metrics` endpoint handler returning Prometheus text format
//! - Request metrics middleware (count + latency)
//! - A dedicated registry for service-level HTTP metrics

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

/// Global Prometheus registry for HTTP metrics.
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// HTTP request counter metric.
///
/// Labels:
/// - `method`: HTTP method (GET, POST, etc.)
/// - `path`: Request path
/// - `status`: HTTP status code
static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let opts = Opts::new("http_requests_total", "Total number of HTTP requests processed");
    let counter = IntCounterVec::new(opts, &["method", "path", "status"])
        .expect("failed to create http_requests_total metric");

    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("failed to register http_requests_total metric");

    counter
});

/// HTTP request duration histogram.
///
/// Labels:
/// - `method`: HTTP method
/// - `path`: Request path
/// - `status_code`: HTTP status code
static HTTP_REQUEST_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    let opts =
        HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);

    let histogram = HistogramVec::new(opts, &["method", "path", "status_code"])
        .expect("failed to create http_request_duration_seconds metric");

    REGISTRY
        .register(Box::new(histogram.clone()))
        .expect("failed to register http_request_duration_seconds metric");

    histogram
});

/// Ensure metric vectors are initialized and registered with the global registry.
fn ensure_metrics_registered() {
    Lazy::force(&HTTP_REQUESTS_TOTAL);
    Lazy::force(&HTTP_REQUEST_DURATION_SECONDS);
}

/// Prometheus metrics endpoint handler.
pub async fn metrics_handler() -> impl IntoResponse {
    ensure_metrics_registered();

    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            let metrics_text = String::from_utf8(buffer).unwrap_or_else(|error| {
                tracing::error!(%error, "failed to convert metrics to UTF-8");
                String::from("# Error: failed to encode metrics")
            });
            (StatusCode::OK, metrics_text).into_response()
        }
        Err(error) => {
            tracing::error!(%error, "failed to encode metrics");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response()
        }
    }
}

/// Middleware that records request count and latency.
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    let elapsed = start.elapsed();

    HTTP_REQUESTS_TOTAL.with_label_values(&[&method, &path, &status]).inc();
    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &path, &status])
        .observe(elapsed.as_secs_f64());

    tracing::debug!(
        %method,
        %path,
        %status,
        elapsed_ms = elapsed.as_millis(),
        "recorded HTTP request metrics"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, response::IntoResponse, routing::get};
    use tower::ServiceExt;

    #[test]
    fn metrics_counter_is_registered() {
        HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/test", "200"]).inc();

        let metric_families = REGISTRY.gather();
        let counter = metric_families.iter().find(|family| family.name() == "http_requests_total");

        assert!(counter.is_some(), "http_requests_total should be registered");
    }

    #[tokio::test]
    async fn metrics_handler_returns_metrics_text() {
        // Ensure the metric has at least one observation so it appears in the output
        HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/test-metrics-handler", "200"]).inc();

        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_text = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_text.contains("http_requests_total"));
    }

    #[tokio::test]
    async fn metrics_middleware_records_request_labels() {
        async fn ok_handler() -> StatusCode {
            StatusCode::NO_CONTENT
        }

        let app = Router::new()
            .route("/metrics-test-path", get(ok_handler))
            .layer(axum::middleware::from_fn(metrics_middleware));

        let response = app
            .oneshot(Request::builder().uri("/metrics-test-path").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let metrics_response = metrics_handler().await.into_response();
        let body = axum::body::to_bytes(metrics_response.into_body(), usize::MAX).await.unwrap();
        let body_text = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_text.contains("http_requests_total"));
        assert!(body_text.contains("path=\"/metrics-test-path\""));
        assert!(body_text.contains("method=\"GET\""));
        assert!(body_text.contains("status=\"204\""));
    }
}
