use axum::http::{Method, Request};
use axum::{body::Body, extract::State, http::StatusCode, middleware::Next, response::Response};

use crate::{AppError, AppState, ErrorCode};

pub const PLATFORM_AUTH_HEADER: &str = "x-platform-token";

/// Enforces platform auth for write endpoints when PLATFORM_AUTH_MODE=basic.
pub async fn platform_auth_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !state.platform_auth.requires_auth() {
        return Ok(next.run(request).await);
    }

    if matches!(request.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(request).await);
    }

    let provided = request.headers().get(PLATFORM_AUTH_HEADER).and_then(|v| v.to_str().ok());

    if state.platform_auth.is_authorized(provided) {
        return Ok(next.run(request).await);
    }

    Err(AppError::new(
        StatusCode::UNAUTHORIZED,
        ErrorCode::Unauthorized,
        "Unauthorized: missing or invalid platform token",
    ))
}
