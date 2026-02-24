//! Request-ID primitives for HTTP middleware.
//!
//! This crate intentionally encapsulates a single concern:
//! extracting or generating request correlation IDs, applying them to the request
//! context, and echoing them back in the response.

#![forbid(unsafe_code)]

use std::fmt;

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

/// Header name used for request correlation.
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Typed request identifier propagated through request extensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(String);

impl RequestId {
    /// Create a request ID from a concrete value.
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a fresh v4 UUID-style request ID.
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Borrow the request identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Extract request ID from headers, or generate one when absent/invalid.
pub fn extract_request_id(headers: &HeaderMap) -> RequestId {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| RequestId::new(value.to_string()))
        .unwrap_or_else(RequestId::generate)
}

/// Request-ID middleware.
///
/// The middleware:
/// 1. Extracts or generates a request ID
/// 2. Records it in the active tracing span
/// 3. Stores it in request extensions
/// 4. Mirrors it into response headers
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = extract_request_id(request.headers());

    Span::current().record("request_id", request_id.as_str());
    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;

    if let Ok(header_value) = HeaderValue::from_str(request_id.as_str()) {
        response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
    }

    response
}

/// Layer constructor mirror for callers that register middleware directly.
pub fn request_id_layer() -> impl tower::Layer<axum::routing::Route> + Clone {
    axum::middleware::from_fn::<_, ()>(request_id_middleware)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, routing::get};
    use tower::util::ServiceExt;

    #[test]
    fn request_id_display() {
        let request_id = RequestId::new("test-123".to_string());
        assert_eq!(format!("{}", request_id), "test-123");
    }

    #[test]
    fn request_id_as_str() {
        let request_id = RequestId::new("test-456".to_string());
        assert_eq!(request_id.as_str(), "test-456");
    }

    #[test]
    fn request_id_generate_returns_a_uuid() {
        let request_id = RequestId::generate();
        let parsed = Uuid::parse_str(request_id.as_str());
        assert!(parsed.is_ok());
    }

    #[test]
    fn extract_request_id_uses_header_when_present() {
        let mut headers = HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_static("trace-from-header"));

        assert_eq!(extract_request_id(&headers).as_str(), "trace-from-header");
    }

    #[test]
    fn extract_request_id_generates_when_missing() {
        let request_id = extract_request_id(&HeaderMap::new());
        assert!(Uuid::parse_str(request_id.as_str()).is_ok());
    }

    #[tokio::test]
    async fn middleware_echoes_existing_request_id() {
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let request = Request::builder()
            .uri("/")
            .header(REQUEST_ID_HEADER, "existing-id")
            .body(Body::empty())
            .expect("valid request body");

        let response = app.oneshot(request).await.expect("response");
        let echoed = response.headers().get(REQUEST_ID_HEADER).expect("request ID echoed");

        assert_eq!(echoed, "existing-id");
    }

    #[tokio::test]
    async fn middleware_generates_request_id_when_absent() {
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let request = Request::builder().uri("/").body(Body::empty()).expect("valid request body");
        let response = app.oneshot(request).await.expect("response");
        let echoed = response.headers().get(REQUEST_ID_HEADER).expect("request ID echoed");
        let text = echoed.to_str().expect("valid header value");
        assert!(Uuid::parse_str(text).is_ok());
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn header_id() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._-]{1,48}".prop_map(|value| value.to_string())
        }

        proptest! {
            #[test]
            fn prop_request_id_prefers_header(header in header_id()) {
                let mut headers = HeaderMap::new();
                headers.insert(REQUEST_ID_HEADER, HeaderValue::from_str(&header).unwrap());

                let extracted = extract_request_id(&headers);
                prop_assert_eq!(extracted.as_str(), header.as_str());
            }

            #[test]
            fn prop_request_id_generates_uuid_when_missing(_seed: u8) {
                let generated = extract_request_id(&HeaderMap::new());
                prop_assert!(Uuid::parse_str(generated.as_str()).is_ok());
            }
        }
    }
}
