//! HTTP error types and error mapping traits
//!
//! This crate provides canonical error types and error mapping traits for HTTP handlers.
//! It is designed to be framework-agnostic where possible, with optional axum support.
//!
//! # Design Philosophy
//!
//! Errors should be:
//! - **Actionable**: Include enough context to debug issues
//! - **Structured**: Use typed fields instead of string concatenation
//! - **Correlated**: Include request ID for tracing
//! - **Secure**: Don't leak internal details to clients
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use http_errors::{ErrorCode, HttpError};
//!
//! // Create a simple error
//! let error = HttpError::bad_request("Invalid input");
//!
//! // Create an error with code and context
//! let error = HttpError::validation_error(
//!     ErrorCode::InvalidFormat,
//!     "Task ID must match pattern TASK-\\d+"
//! );
//! ```

use serde::Serialize;

// ============================================================================
// Error Codes
// ============================================================================

/// Machine-readable error codes
///
/// These codes allow clients to programmatically handle different error scenarios
/// without parsing error messages. They also help with:
/// - Metrics aggregation (count errors by code)
/// - Alert rules (alert on specific error codes)
/// - Client-side error handling (show appropriate UI based on code)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors (4xx)
    InvalidRequest,
    InvalidAmount,
    MissingField,
    InvalidFormat,
    Unauthorized,

    // Business logic errors (4xx)
    ResourceNotFound,
    InvalidState,
    InvalidTransition,
    Conflict,
    DuplicateRequest,

    // System errors (5xx)
    InternalError,
    ServiceUnavailable,
    DatabaseError,
    ExternalServiceError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InvalidRequest => write!(f, "INVALID_REQUEST"),
            ErrorCode::InvalidAmount => write!(f, "INVALID_AMOUNT"),
            ErrorCode::MissingField => write!(f, "MISSING_FIELD"),
            ErrorCode::InvalidFormat => write!(f, "INVALID_FORMAT"),
            ErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            ErrorCode::ResourceNotFound => write!(f, "RESOURCE_NOT_FOUND"),
            ErrorCode::InvalidState => write!(f, "INVALID_STATE"),
            ErrorCode::InvalidTransition => write!(f, "INVALID_TRANSITION"),
            ErrorCode::Conflict => write!(f, "CONFLICT"),
            ErrorCode::DuplicateRequest => write!(f, "DUPLICATE_REQUEST"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
            ErrorCode::ServiceUnavailable => write!(f, "SERVICE_UNAVAILABLE"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ExternalServiceError => write!(f, "EXTERNAL_SERVICE_ERROR"),
        }
    }
}

// ============================================================================
// Error Response DTO
// ============================================================================

/// Standard error response DTO matching platform contract
///
/// This format is required for consistent error handling across all platform endpoints.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., "not_found", "spec_load_error")
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Request ID for correlation
    #[serde(rename = "requestId")]
    pub request_id: String,
}

// ============================================================================
// HttpError - Core Error Type
// ============================================================================

/// HTTP error with status code and error code
///
/// This error type provides a minimal, framework-agnostic representation
/// of HTTP errors that can be converted to framework-specific responses.
#[derive(Debug)]
pub struct HttpError {
    /// HTTP status code to return
    pub status: u16,
    /// Machine-readable error code
    pub code: ErrorCode,
    /// User-facing error message (safe to expose)
    pub message: String,
}

impl HttpError {
    /// Create a new error with status, code, and message
    pub fn new(status: u16, code: ErrorCode, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into() }
    }

    /// Create a bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, ErrorCode::InvalidRequest, message)
    }

    /// Create a validation error (400)
    pub fn validation_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(400, code, message)
    }

    /// Create a business logic error (422 Unprocessable Entity)
    pub fn business_logic_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(422, code, message)
    }

    /// Create a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, ErrorCode::ResourceNotFound, message)
    }

    /// Create an internal server error (500)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(500, ErrorCode::InternalError, message)
    }

    /// Create an unauthorized error (401)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(401, ErrorCode::Unauthorized, message)
    }

    /// Create a conflict error (409)
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(409, ErrorCode::Conflict, message)
    }

    /// Create a service unavailable error (503)
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(503, ErrorCode::ServiceUnavailable, message)
    }

    /// Create an error response DTO from this error
    pub fn to_response(&self, request_id: impl Into<String>) -> ErrorResponse {
        ErrorResponse {
            error: self.code.to_string(),
            message: self.message.clone(),
            request_id: request_id.into(),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.status, self.code, self.message)
    }
}

impl std::error::Error for HttpError {}

// ============================================================================
// Error Mapping Trait
// ============================================================================

/// Trait for converting domain errors to HTTP errors
///
/// This trait allows domain-specific error types to be converted
/// to HTTP errors with appropriate status codes.
pub trait ToHttpError {
    /// Convert this error to an HTTP error
    fn to_http_error(&self) -> HttpError;
}

// ============================================================================
// Optional Axum Support
// ============================================================================

#[cfg(feature = "axum")]
mod axum_support {
    use super::*;
    use axum::{
        Json,
        http::{HeaderValue, StatusCode, header::HeaderName},
        response::{IntoResponse, Response},
    };

    impl IntoResponse for HttpError {
        fn into_response(self) -> Response {
            let request_id = uuid::Uuid::new_v4().to_string();

            let body = Json(self.to_response(&request_id));

            let mut response = (
                StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response();

            if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                response
                    .headers_mut()
                    .insert(HeaderName::from_static("x-request-id"), header_value);
            }

            response
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidAmount.to_string(), "INVALID_AMOUNT");
        assert_eq!(ErrorCode::ResourceNotFound.to_string(), "RESOURCE_NOT_FOUND");
    }

    #[test]
    fn test_bad_request_error() {
        let error = HttpError::bad_request("Invalid input");
        assert_eq!(error.status, 400);
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn test_not_found_error() {
        let error = HttpError::not_found("Resource not found");
        assert_eq!(error.status, 404);
        assert_eq!(error.code, ErrorCode::ResourceNotFound);
    }

    #[test]
    fn test_internal_error() {
        let error = HttpError::internal_error("Something went wrong");
        assert_eq!(error.status, 500);
        assert_eq!(error.code, ErrorCode::InternalError);
    }

    #[test]
    fn test_error_response_serialization() {
        let error =
            HttpError::validation_error(ErrorCode::InvalidAmount, "Amount must be positive");
        let response = error.to_response("req-123");

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("INVALID_AMOUNT"));
        assert!(json.contains("Amount must be positive"));
        assert!(json.contains("req-123"));
        assert!(json.contains(r#""requestId":"req-123""#));
    }

    #[test]
    fn test_error_display() {
        let error = HttpError::not_found("Task not found");
        let display = format!("{}", error);
        assert!(display.contains("404"));
        assert!(display.contains("RESOURCE_NOT_FOUND"));
        assert!(display.contains("Task not found"));
    }
}
