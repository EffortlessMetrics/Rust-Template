//! Backward-compatible CORS middleware wrapper.
//!
//! Canonical CORS implementation lives in the `http-middleware` microcrate.

use crate::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub use http_middleware::CorsConfig;

/// app-http compatibility wrapper that sources config from application state.
pub async fn cors_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    http_middleware::cors_middleware(state.cors_config.clone(), request, next).await
}
