//! Error handling with observability and correlation
//!
//! This module provides a comprehensive error type that supports:
//! - Machine-readable error codes
//! - AC ID and Feature ID tracking (for product/feature correlation)
//! - Structured logging with correlation fields
//! - Proper HTTP responses with JSON error bodies
//! - Request ID correlation
//! - Error tracking for `/platform/status` surfacing
//!
//! # Design Philosophy
//!
//! Errors should be:
//! 1. **Actionable**: Include enough context to debug issues
//! 2. **Structured**: Use typed fields instead of string concatenation
//! 3. **Correlated**: Include request ID, AC ID, feature ID for tracing
//! 4. **Secure**: Don't leak internal details to clients
//! 5. **Observable**: Log errors with structured data for analysis

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::{HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tracing::{error, warn};

use crate::middleware::request_id::RequestId;

// ============================================================================
// Error Tracking for /platform/status
// ============================================================================

/// Summary of the last error that occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastErrorSummary {
    pub category: String,
    pub message: String,
    pub status_code: u16,
    pub occurred_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Aggregated error statistics for the service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub client_errors: u64,
    pub server_errors: u64,
}

/// Error summary surfaced via `/platform/status`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub has_recent_errors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<LastErrorSummary>,
    pub stats: ErrorStats,
}

static ERROR_TRACKER: OnceLock<Mutex<ErrorSummary>> = OnceLock::new();

fn error_tracker() -> &'static Mutex<ErrorSummary> {
    ERROR_TRACKER.get_or_init(|| Mutex::new(ErrorSummary::default()))
}

fn record_error(code: &ErrorCode, status: StatusCode, message: &str, request_id: Option<&str>) {
    if let Ok(mut tracker) = error_tracker().try_lock() {
        tracker.has_recent_errors = true;
        tracker.stats.total_errors += 1;

        if status.is_client_error() {
            tracker.stats.client_errors += 1;
        } else if status.is_server_error() {
            tracker.stats.server_errors += 1;
        }

        tracker.last_error = Some(LastErrorSummary {
            category: error_code_category(code),
            message: message.to_string(),
            status_code: status.as_u16(),
            occurred_at: Utc::now(),
            request_id: request_id.map(String::from),
        });
    }
}

pub fn get_error_summary() -> ErrorSummary {
    error_tracker().try_lock().map(|t| t.clone()).unwrap_or_default()
}

fn error_code_category(code: &ErrorCode) -> String {
    match code {
        ErrorCode::ResourceNotFound => "resource_not_found".to_string(),
        ErrorCode::InvalidTransition => "invalid_transition".to_string(),
        ErrorCode::InvalidRequest => "invalid_request".to_string(),
        ErrorCode::InvalidAmount => "invalid_amount".to_string(),
        ErrorCode::MissingField => "missing_field".to_string(),
        ErrorCode::InvalidFormat => "invalid_format".to_string(),
        ErrorCode::Unauthorized => "unauthorized".to_string(),
        ErrorCode::InvalidState => "invalid_state".to_string(),
        ErrorCode::Conflict => "conflict".to_string(),
        ErrorCode::DuplicateRequest => "duplicate_request".to_string(),
        ErrorCode::InternalError => "internal_error".to_string(),
        ErrorCode::ServiceUnavailable => "service_unavailable".to_string(),
        ErrorCode::DatabaseError => "database_error".to_string(),
        ErrorCode::ExternalServiceError => "external_service_error".to_string(),
    }
}

// ============================================================================
// Error Codes and AppError
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    InvalidAmount,
    MissingField,
    InvalidFormat,
    Unauthorized,
    ResourceNotFound,
    InvalidState,
    InvalidTransition,
    Conflict,
    DuplicateRequest,
    InternalError,
    ServiceUnavailable,
    DatabaseError,
    ExternalServiceError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Default)]
struct ErrorDetails {
    context: HashMap<String, serde_json::Value>,
    ac_id: Option<String>,
    feature_id: Option<String>,
    request_id: Option<String>,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    code: ErrorCode,
    message: String,
    details: Box<ErrorDetails>,
}

impl AppError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into(), details: Box::default() }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::InvalidRequest, message)
    }

    pub fn validation_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn business_logic_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::ResourceNotFound, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError, message)
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.details.context.insert(key.into(), json_value);
        }
        self
    }

    pub fn with_ac_id(mut self, ac_id: impl Into<String>) -> Self {
        self.details.ac_id = Some(ac_id.into());
        self
    }

    pub fn with_feature_id(mut self, feature_id: impl Into<String>) -> Self {
        self.details.feature_id = Some(feature_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.details.request_id = Some(request_id.into());
        self
    }

    pub fn spec_load_error(context: &str, err: impl std::fmt::Display) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to {}: {}", context, err),
        )
    }

    pub fn io_error(context: &str, err: impl std::fmt::Display) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to {}: {}", context, err),
        )
    }

    fn log_error(&self) {
        let is_server_error = self.status.is_server_error();
        record_error(&self.code, self.status, &self.message, self.details.request_id.as_deref());

        if is_server_error {
            error!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.details.context,
                ac_id = ?self.details.ac_id,
                feature_id = ?self.details.feature_id,
                "Internal server error occurred"
            );
        } else {
            warn!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.details.context,
                ac_id = ?self.details.ac_id,
                feature_id = ?self.details.feature_id,
                "Client error occurred"
            );
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(rename = "requestId")]
    request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feature_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.log_error();
        let request_id =
            self.details.request_id.clone().unwrap_or_else(|| RequestId::generate().to_string());

        let body = Json(ErrorResponse {
            error: self.code.to_string(),
            message: self.message.clone(),
            request_id: request_id.clone(),
            ac_id: self.details.ac_id.clone(),
            feature_id: self.details.feature_id.clone(),
        });

        let mut response = (self.status, body).into_response();
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
        }
        response
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::validation_error(
            ErrorCode::InvalidRequest,
            format!("Invalid JSON: {}", rejection),
        )
    }
}

impl From<spec_runtime::ValidationError> for AppError {
    fn from(error: spec_runtime::ValidationError) -> Self {
        use spec_runtime::ValidationError::*;
        match error {
            InvalidFormat { .. } | TooLong { .. } | InvalidCharacters { .. } | TooDeep { .. } => {
                AppError::validation_error(ErrorCode::InvalidFormat, error.to_string())
            }
            EmptyField(_) => AppError::validation_error(ErrorCode::MissingField, error.to_string()),
        }
    }
}

impl From<spec_runtime::SpecError> for AppError {
    fn from(error: spec_runtime::SpecError) -> Self {
        use spec_runtime::SpecError::*;
        match error {
            LedgerLoad(msg) | Parse(msg) | ConfigValidation(msg) => {
                AppError::spec_load_error("load spec", msg)
            }
            Io { path, source } => AppError::io_error(&format!("read {}", path.display()), source),
            Yaml(e) => AppError::spec_load_error("parse YAML", e),
            Internal(msg) => AppError::internal_error(msg),
        }
    }
}

impl From<business_core::ports::RepositoryError> for AppError {
    fn from(error: business_core::ports::RepositoryError) -> Self {
        use business_core::ports::RepositoryError::*;
        match error {
            NotFound(msg) => AppError::not_found(msg),
            Database(msg) => {
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::DatabaseError, msg)
            }
            Serialization(msg) => AppError::internal_error(format!("Serialization error: {}", msg)),
            Io(e) => AppError::io_error("repository I/O", e),
            Other(msg) => AppError::internal_error(msg),
        }
    }
}

impl From<business_core::governance::GovernanceError> for AppError {
    fn from(error: business_core::governance::GovernanceError) -> Self {
        use business_core::governance::GovernanceError::*;
        match error {
            TaskNotFound(id) => AppError::not_found(format!("Task not found: {:?}", id)),
            InvalidTransition { from, to } => AppError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidTransition,
                format!("Invalid status transition from {} to {}", from, to),
            ),
            Lock(msg) => AppError::internal_error(format!("Lock error: {}", msg)),
            Io(e) => AppError::internal_error(format!("IO error: {}", e)),
            Serialization(msg) => AppError::internal_error(format!("Serialization error: {}", msg)),
        }
    }
}

impl From<gov_http::PlatformError> for AppError {
    fn from(error: gov_http::PlatformError) -> Self {
        use gov_http::PlatformError::*;
        match error {
            NotFound(msg) => AppError::not_found(msg),
            TooManyEntries(actual, max) => AppError::internal_error(format!(
                "Collection size limit exceeded: {} > {}",
                actual, max
            )),
            SpecLoad { context, source } => AppError::spec_load_error(context, source),
            Internal(msg) => AppError::internal_error(msg),
        }
    }
}
