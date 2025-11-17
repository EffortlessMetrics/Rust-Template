//! Error handling with observability and correlation
//!
//! This module provides a comprehensive error type that supports:
//! - Machine-readable error codes
//! - AC ID and Feature ID tracking (for product/feature correlation)
//! - Structured logging with correlation fields
//! - Proper HTTP responses with JSON error bodies
//! - Request ID correlation
//!
//! # Design Philosophy
//!
//! Errors should be:
//! 1. **Actionable**: Include enough context to debug issues
//! 2. **Structured**: Use typed fields instead of string concatenation
//! 3. **Correlated**: Include request ID, AC ID, feature ID for tracing
//! 4. **Secure**: Don't leak internal details to clients
//! 5. **Observable**: Log errors with structured data for analysis
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use crate::errors::{AppError, ErrorCode};
//!
//! // Simple error
//! return Err(AppError::bad_request("Invalid amount"));
//!
//! // Error with code and context
//! return Err(AppError::validation_error(
//!     ErrorCode::InvalidAmount,
//!     "Amount must be greater than 0"
//! ).with_context("amount_cents", payload.amount_cents));
//!
//! // Error with AC/Feature tracking
//! return Err(AppError::business_logic_error(
//!     ErrorCode::RefundNotEligible,
//!     "Order not eligible for refund"
//! ).with_ac_id("AC-123")
//!   .with_feature_id("FT-456"));
//! ```

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::{HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, warn};

use crate::middleware::request_id::RequestId;

/// Machine-readable error codes
///
/// These codes allow clients to programmatically handle different error scenarios
/// without parsing error messages. They also help with:
/// - Metrics aggregation (count errors by code)
/// - Alert rules (alert on specific error codes)
/// - Client-side error handling (show appropriate UI based on code)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors (4xx)
    InvalidRequest,
    InvalidAmount,
    MissingField,
    InvalidFormat,

    // Business logic errors (4xx)
    RefundNotEligible,
    OrderNotFound,
    InsufficientFunds,
    DuplicateRequest,

    // System errors (5xx)
    InternalError,
    ServiceUnavailable,
    DatabaseError,
    ExternalServiceError,
    // Add more as needed for your domain
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InvalidRequest => write!(f, "INVALID_REQUEST"),
            ErrorCode::InvalidAmount => write!(f, "INVALID_AMOUNT"),
            ErrorCode::MissingField => write!(f, "MISSING_FIELD"),
            ErrorCode::InvalidFormat => write!(f, "INVALID_FORMAT"),
            ErrorCode::RefundNotEligible => write!(f, "REFUND_NOT_ELIGIBLE"),
            ErrorCode::OrderNotFound => write!(f, "ORDER_NOT_FOUND"),
            ErrorCode::InsufficientFunds => write!(f, "INSUFFICIENT_FUNDS"),
            ErrorCode::DuplicateRequest => write!(f, "DUPLICATE_REQUEST"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
            ErrorCode::ServiceUnavailable => write!(f, "SERVICE_UNAVAILABLE"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ExternalServiceError => write!(f, "EXTERNAL_SERVICE_ERROR"),
        }
    }
}

/// Application error with full observability support
///
/// This error type includes:
/// - HTTP status code (for response)
/// - Error code (for clients and metrics)
/// - User message (safe to show to clients)
/// - Internal context (for logging, not shown to clients)
/// - AC ID and Feature ID (for product tracking)
/// - Request ID (for correlation - AC-TPL-004)
#[derive(Debug)]
pub struct AppError {
    /// HTTP status code to return
    status: StatusCode,
    /// Machine-readable error code
    code: ErrorCode,
    /// User-facing error message (safe to expose)
    message: String,
    /// Internal context for debugging (logged but not exposed to clients)
    context: HashMap<String, serde_json::Value>,
    /// AC (Acceptance Criteria) ID for tracking features
    ac_id: Option<String>,
    /// Feature ID for tracking which feature this relates to
    feature_id: Option<String>,
    /// Request ID for correlation (AC-TPL-004)
    /// If None, a new UUID will be generated when converting to response
    request_id: Option<String>,
}

impl AppError {
    /// Create a new error with status, code, and message
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            context: HashMap::new(),
            ac_id: None,
            feature_id: None,
            request_id: None,
        }
    }

    /// Create a bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::InvalidRequest, message)
    }

    /// Create a validation error (400)
    pub fn validation_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    /// Create a business logic error (422 Unprocessable Entity)
    pub fn business_logic_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    /// Create a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::OrderNotFound, message)
    }

    /// Create an internal server error (500)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError, message)
    }

    /// Add context field for debugging
    ///
    /// Context is logged but not exposed to clients
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }

    /// Add AC (Acceptance Criteria) ID
    ///
    /// Used to track which acceptance criteria this error relates to
    pub fn with_ac_id(mut self, ac_id: impl Into<String>) -> Self {
        self.ac_id = Some(ac_id.into());
        self
    }

    /// Add Feature ID
    ///
    /// Used to track which feature this error relates to
    pub fn with_feature_id(mut self, feature_id: impl Into<String>) -> Self {
        self.feature_id = Some(feature_id.into());
        self
    }

    /// Add Request ID (AC-TPL-004)
    ///
    /// Used for distributed tracing and correlation.
    /// If not set, a UUID will be generated automatically.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Log the error with structured fields
    fn log_error(&self) {
        // Determine if this is a client error (4xx) or server error (5xx)
        let is_server_error = self.status.is_server_error();

        // Create structured log event
        if is_server_error {
            // Server errors are more severe - log as error level
            error!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.context,
                ac_id = ?self.ac_id,
                feature_id = ?self.feature_id,
                "Internal server error occurred"
            );
        } else {
            // Client errors are expected - log as warn level
            warn!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.context,
                ac_id = ?self.ac_id,
                feature_id = ?self.feature_id,
                "Client error occurred"
            );
        }
    }
}

/// JSON error response body
///
/// This is what clients receive when an error occurs.
/// Matches the ErrorResponse schema in openapi.yaml (AC-TPL-003).
#[derive(Debug, Serialize)]
struct ErrorResponse {
    /// Machine-readable error code (required by AC-TPL-003)
    error: String,
    /// Human-readable error message (required by AC-TPL-003)
    message: String,
    /// Request ID for correlation (required by AC-TPL-003, AC-TPL-004)
    #[serde(rename = "requestId")]
    request_id: String,
    /// Optional AC ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_id: Option<String>,
    /// Optional Feature ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    feature_id: Option<String>,
    // Note: context is NOT included (internal only)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error with full context
        self.log_error();

        // Get or generate request ID (AC-TPL-004)
        let request_id =
            self.request_id.clone().unwrap_or_else(|| RequestId::generate().to_string());

        // Create client-safe response matching ErrorResponse schema (AC-TPL-003)
        let body = Json(ErrorResponse {
            error: self.code.to_string(),
            message: self.message.clone(),
            request_id: request_id.clone(),
            ac_id: self.ac_id.clone(),
            feature_id: self.feature_id.clone(),
        });

        // Create response with status code
        let mut response = (self.status, body).into_response();

        // Add X-Request-ID header (AC-TPL-004)
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
        }

        response
    }
}

/// Convert JSON rejection errors to AppError
///
/// This allows us to handle JSON parsing errors consistently
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::validation_error(
            ErrorCode::InvalidRequest,
            format!("Invalid JSON: {}", rejection),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidAmount.to_string(), "INVALID_AMOUNT");
        assert_eq!(ErrorCode::RefundNotEligible.to_string(), "REFUND_NOT_ELIGIBLE");
    }

    #[test]
    fn test_bad_request_error() {
        let error = AppError::bad_request("Invalid input");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn test_error_with_context() {
        let error = AppError::validation_error(ErrorCode::InvalidAmount, "Amount must be positive")
            .with_context("amount", -100)
            .with_context("field", "amount_cents");

        assert!(error.context.contains_key("amount"));
        assert!(error.context.contains_key("field"));
    }

    #[test]
    fn test_error_with_ac_and_feature() {
        let error =
            AppError::business_logic_error(ErrorCode::RefundNotEligible, "Order not refundable")
                .with_ac_id("AC-123")
                .with_feature_id("FT-456");

        assert_eq!(error.ac_id, Some("AC-123".to_string()));
        assert_eq!(error.feature_id, Some("FT-456".to_string()));
    }

    #[test]
    fn test_error_serialization() {
        let error = AppError::validation_error(ErrorCode::InvalidAmount, "Amount must be positive")
            .with_ac_id("AC-123")
            .with_feature_id("FT-456")
            .with_request_id("req-test-123");

        let response = ErrorResponse {
            error: error.code.to_string(),
            message: error.message.clone(),
            request_id: error.request_id.clone().unwrap_or_default(),
            ac_id: error.ac_id.clone(),
            feature_id: error.feature_id.clone(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("INVALID_AMOUNT"));
        assert!(json.contains("Amount must be positive"));
        assert!(json.contains("req-test-123"));
        assert!(json.contains("AC-123"));
        assert!(json.contains("FT-456"));
        // Verify it uses "error" not "code" (AC-TPL-003)
        assert!(json.contains(r#""error":"INVALID_AMOUNT""#));
        // Verify it uses "requestId" not "request_id" (AC-TPL-003)
        assert!(json.contains(r#""requestId":"req-test-123""#));
    }
}
