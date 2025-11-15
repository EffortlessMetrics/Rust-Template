//! Request ID correlation middleware
//!
//! This middleware implements distributed tracing correlation by:
//! 1. Reading X-Request-ID header from incoming requests (if present)
//! 2. Generating a new UUID if no request ID is provided
//! 3. Storing the ID in request extensions for handler access
//! 4. Adding the ID to the tracing span for log correlation
//! 5. Including it in the response header for client tracking
//!
//! # Observability Story
//!
//! Request IDs enable:
//! - **Distributed Tracing**: Track a request across multiple services
//! - **Log Correlation**: Group all logs for a single request
//! - **Debugging**: Clients can provide request IDs when reporting issues
//! - **Metrics**: Correlate metrics with specific requests
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use axum::Router;
//! use crate::middleware::request_id::RequestIdLayer;
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(RequestIdLayer);
//! ```
//!
//! # Handler Access
//!
//! ```rust,ignore
//! use axum::extract::Extension;
//! use crate::middleware::request_id::RequestId;
//!
//! async fn handler(Extension(request_id): Extension<RequestId>) -> impl IntoResponse {
//!     info!(request_id = %request_id, "Processing request");
//!     // ... handler logic
//! }
//! ```

use axum::{
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

/// Header name for request ID (standard practice)
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Typed wrapper for request ID
///
/// This newtype provides type safety and makes it clear when we're working
/// with request IDs vs arbitrary strings.
#[derive(Debug, Clone)]
pub struct RequestId(String);

impl RequestId {
    /// Create a new request ID from a string
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new random request ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the request ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Request ID middleware implementation
///
/// This is the core middleware function that:
/// 1. Extracts or generates a request ID
/// 2. Adds it to the tracing span
/// 3. Stores it in request extensions
/// 4. Adds it to the response headers
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Step 1: Extract request ID from header or generate a new one
    let request_id = extract_or_generate_request_id(&request);

    // Step 2: Record the request ID in the current tracing span
    // This ensures all logs within this request context include the request_id field
    Span::current().record("request_id", request_id.as_str());

    // Step 3: Store request ID in request extensions
    // This allows handlers to access the request ID via Extension<RequestId>
    request.extensions_mut().insert(request_id.clone());

    // Step 4: Process the request through the handler chain
    let mut response = next.run(request).await;

    // Step 5: Add request ID to response headers
    // This allows clients to correlate responses with their requests
    if let Ok(header_value) = HeaderValue::from_str(request_id.as_str()) {
        response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
    }

    response
}

/// Extract request ID from header or generate a new one
fn extract_or_generate_request_id(request: &Request) -> RequestId {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| RequestId::new(s.to_string()))
        .unwrap_or_else(RequestId::generate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_display() {
        let id = RequestId::new("test-123".to_string());
        assert_eq!(format!("{}", id), "test-123");
    }

    #[test]
    fn test_request_id_as_str() {
        let id = RequestId::new("test-456".to_string());
        assert_eq!(id.as_str(), "test-456");
    }

    #[test]
    fn test_request_id_generate() {
        let id1 = RequestId::generate();
        let id2 = RequestId::generate();
        // Generated IDs should be different
        assert_ne!(id1.as_str(), id2.as_str());
        // Should be valid UUIDs
        assert!(Uuid::parse_str(id1.as_str()).is_ok());
        assert!(Uuid::parse_str(id2.as_str()).is_ok());
    }

    #[test]
    fn test_extract_or_generate_from_empty_request() {
        use axum::body::Body;
        use axum::http::Request;

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let request_id = extract_or_generate_request_id(&request);
        // Should be a valid UUID since no header was provided
        assert!(Uuid::parse_str(request_id.as_str()).is_ok());
    }

    #[test]
    fn test_extract_or_generate_from_request_with_header() {
        use axum::body::Body;
        use axum::http::Request;

        let test_id = "test-request-id-12345";
        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, test_id)
            .body(Body::empty())
            .unwrap();

        let request_id = extract_or_generate_request_id(&request);
        assert_eq!(request_id.as_str(), test_id);
    }
}
