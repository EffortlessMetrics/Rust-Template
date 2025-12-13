//! Error types for gov-http handlers.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Error response DTO.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Platform API error type.
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
            PlatformError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg.clone())
            }
        };

        let body = Json(ErrorResponse { error: error_type.to_string(), message });

        (status, body).into_response()
    }
}
