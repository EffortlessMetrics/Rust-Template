//! Backward-compatible metrics exports.
//!
//! Metrics implementation now lives in the dedicated `http-metrics` microcrate.
//! Keep this module so `app_http::metrics::*` imports continue to work.

pub use http_metrics::{metrics_handler, metrics_middleware};
