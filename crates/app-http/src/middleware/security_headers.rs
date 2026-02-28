//! Backward-compatible security headers middleware wrapper.
//!
//! Canonical security headers implementation lives in the `http-middleware` microcrate.

use crate::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub use http_middleware::SecurityHeadersConfig;

/// app-http compatibility wrapper that sources config from application state.
pub async fn security_headers_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    state.security_headers_config.apply_headers(&mut response);
    response
}
