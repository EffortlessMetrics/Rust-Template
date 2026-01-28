//! Common extractors for gov-http handlers.
//!
//! This module provides reusable extractors for:
//! - Request ID extraction and propagation
//! - Auth context extraction (placeholder for future use)

use axum::extract::FromRequestParts;
use http::request::Parts;
use uuid::Uuid;

/// Request ID extractor.
///
/// Extracts or generates a request ID from incoming requests.
/// This is used for request tracing and correlation (AC-TPL-004).
///
/// The request ID is extracted from the `x-request-id` header if present,
/// otherwise a new UUID is generated.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Create a new RequestId with the given value.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the request ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the request ID as a string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract from x-request-id header
        let request_id = parts
            .headers
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Ok(RequestId(request_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_new() {
        let id = RequestId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
    }

    #[test]
    fn test_request_id_into_string() {
        let id = RequestId::new("test-456");
        assert_eq!(id.into_string(), "test-456");
    }
}
