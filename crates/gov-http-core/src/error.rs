//! Error types for gov-http handlers.
//!
//! Error responses follow the platform contract (AC-TPL-ERROR-MAPPING):
//! - `error`: Machine-readable error code
//! - `message`: Human-readable error message
//! - `requestId`: Unique request ID for correlation

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

/// Platform API error type.
///
/// This error type is used across all gov-http-* subrouter crates
/// for consistent error handling.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// Spec file loading error.
    #[error("Failed to load {context}: {source}")]
    SpecLoad {
        context: &'static str,
        #[source]
        source: anyhow::Error,
    },

    /// Resource not found.
    #[error("{0}")]
    NotFound(String),

    /// Too many entries in collection.
    #[error("Too many entries: {0} > {1}")]
    TooManyEntries(usize, usize),

    /// Internal error.
    #[error("{0}")]
    Internal(String),
}

impl PlatformError {
    /// Create a spec load error.
    pub fn spec_load(context: &'static str, source: impl Into<anyhow::Error>) -> Self {
        Self::SpecLoad { context, source: source.into() }
    }

    /// Create a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Create a too many entries error.
    pub fn too_many_entries(actual: usize, max: usize) -> Self {
        Self::TooManyEntries(actual, max)
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl IntoResponse for PlatformError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            PlatformError::SpecLoad { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "spec_load_error", self.to_string())
            }
            PlatformError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            PlatformError::TooManyEntries(_, _) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "too_many_entries", self.to_string())
            }
            PlatformError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg.clone())
            }
        };

        // Generate request ID for correlation (AC-TPL-004)
        let request_id = Uuid::new_v4().to_string();

        let body = Json(ErrorResponse { error: error_type.to_string(), message, request_id });

        (status, body).into_response()
    }
}

/// Error response DTO matching platform contract.
///
/// This format is required by AC-TPL-ERROR-MAPPING for consistent
/// error handling across all platform endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., "not_found", "spec_load_error")
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Request ID for correlation (AC-TPL-004)
    #[serde(rename = "requestId")]
    pub request_id: String,
}

/// Trait for converting domain errors to PlatformError.
///
/// This allows subrouter crates to define their own error types
/// that can be converted to the common PlatformError type.
pub trait ToPlatformError {
    /// Convert this error to a PlatformError.
    fn to_platform_error(&self) -> PlatformError;
}
