//! Request ID primitives shared across HTTP middleware.

use http::HeaderMap;
use uuid::Uuid;

/// Header name for request ID (standard practice).
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Typed wrapper for request ID.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RequestId(String);

impl RequestId {
    /// Create a new request ID from a string.
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new random request ID.
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the request ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Extract request ID from header map or generate a new one.
pub fn from_headers_or_generate(headers: &HeaderMap) -> RequestId {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| RequestId::new(s.to_string()))
        .unwrap_or_else(RequestId::generate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_display() {
        let id = RequestId::new("test-123".to_string());
        assert_eq!(format!("{}", id), "test-123");
    }

    #[test]
    fn request_id_as_str() {
        let id = RequestId::new("test-456".to_string());
        assert_eq!(id.as_str(), "test-456");
    }

    #[test]
    fn request_id_generate() {
        let id1 = RequestId::generate();
        let id2 = RequestId::generate();
        assert_ne!(id1.as_str(), id2.as_str());
        assert!(Uuid::parse_str(id1.as_str()).is_ok());
        assert!(Uuid::parse_str(id2.as_str()).is_ok());
    }

    #[test]
    fn from_headers_extracts_when_present() {
        let mut headers = HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, "provided-id".parse().expect("valid header value"));

        let request_id = from_headers_or_generate(&headers);
        assert_eq!(request_id.as_str(), "provided-id");
    }

    #[test]
    fn from_headers_generates_when_missing() {
        let headers = HeaderMap::new();

        let request_id = from_headers_or_generate(&headers);
        assert!(Uuid::parse_str(request_id.as_str()).is_ok());
    }
}
