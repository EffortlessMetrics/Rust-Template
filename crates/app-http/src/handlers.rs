use crate::{AppError, ErrorCode, RequestId};
use axum::{Extension, Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

// ============================================================================
// Template Core Handlers - Keep these in your service
// ============================================================================

/// Health check endpoint
///
/// Demonstrates:
/// - Accessing request ID from extensions
/// - Basic instrumentation
/// - Simple JSON response
#[instrument(skip(_request_id))]
pub(crate) async fn health(Extension(_request_id): Extension<RequestId>) -> impl IntoResponse {
    // Log with request_id automatically included from span
    info!("Health check requested");

    Json(HealthResponse { status: "ok".to_string(), service: "service-api".to_string() })
}

/// Version information endpoint
#[instrument]
pub(crate) async fn version() -> impl IntoResponse {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

/// Echo endpoint - Used for testing error handling
///
/// Demonstrates:
/// - Validation errors with error codes
/// - Request ID propagation through error responses
/// - ErrorResponse envelope structure
#[instrument(skip(request_id, payload))]
pub(crate) async fn echo(
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<EchoRequest>,
) -> Result<Json<EchoResponse>, AppError> {
    info!("Echo request received");

    // Validation: message cannot be empty
    if payload.message.is_empty() {
        return Err(AppError::validation_error(ErrorCode::MissingField, "Message cannot be empty")
            .with_context("field", "message")
            .with_ac_id("AC-TPL-003") // Links to error envelope AC
            .with_request_id(request_id.as_str())); // AC-TPL-004: Propagate request ID
    }

    Ok(Json(EchoResponse { message: payload.message }))
}

// ============================================================================
// DTOs - Request/Response types for HTTP boundary
// ============================================================================

// Template Core DTOs
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Debug, Serialize)]
struct VersionInfo {
    version: String,
    #[serde(rename = "gitSha")]
    git_sha: String,
}

// Echo endpoint DTOs (used for testing error handling)
#[derive(Debug, Deserialize)]
pub(crate) struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct EchoResponse {
    pub message: String,
}
