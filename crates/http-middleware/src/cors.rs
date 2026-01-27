//! CORS (Cross-Origin Resource Sharing) middleware
//!
//! This module provides CORS configuration to control cross-origin requests
//! and prevent unauthorized access from different domains.
//!
//! # Design Philosophy
//!
//! CORS should be:
//! - **Explicit**: Clearly define allowed origins, methods, headers
//! - **Configurable**: Support wildcard and subdomain patterns
//! - **Environment-aware**: Different settings for dev vs prod
//! - **Secure by default**: Deny by default, allow explicitly

use axum::{
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

/// CORS configuration structure
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsConfig {
    /// List of allowed origins (e.g., ["https://example.com", "http://localhost:3000"])
    pub allowed_origins: Vec<String>,

    /// List of allowed HTTP methods
    pub allowed_methods: Vec<String>,

    /// List of allowed headers
    pub allowed_headers: Vec<String>,

    /// List of exposed headers for clients
    pub exposed_headers: Vec<String>,

    /// Whether credentials are allowed
    pub allow_credentials: bool,

    /// Max age for preflight requests (in seconds)
    pub max_age: Option<u64>,

    /// Whether CORS is enabled
    pub enabled: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
                "PATCH".to_string(),
            ],
            allowed_headers: vec![
                "authorization".to_string(),
                "content-type".to_string(),
                "x-request-id".to_string(),
                "accept".to_string(),
                "origin".to_string(),
            ],
            exposed_headers: vec!["x-request-id".to_string()],
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
            enabled: true,
        }
    }
}

impl CorsConfig {
    /// Create development-friendly configuration
    ///
    /// Allows localhost origins with permissive settings.
    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
            ],
            ..Default::default()
        }
    }

    /// Create production-optimized configuration
    ///
    /// Requires explicit origin configuration.
    pub fn production(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins, allow_credentials: false, ..Default::default() }
    }

    /// Check if origin is allowed
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|allowed| {
            if allowed == "*" {
                return true;
            }
            if allowed == origin {
                return true;
            }
            // Handle path wildcards like "https://example.com/*"
            if allowed.ends_with("/*") && origin.starts_with(&allowed[..allowed.len() - 2]) {
                return true;
            }
            // Handle subdomain wildcards like "https://*.example.com"
            if let Some(pos) = allowed.find("*.")
                && (pos == 0
                    || (pos == 8 && allowed.starts_with("https://"))
                    || (pos == 7 && allowed.starts_with("http://")))
            {
                let wildcard_domain = &allowed[pos + 2..];
                // Origin scheme must match allowed scheme for wildcard matching
                let schemes_match = (origin.starts_with("https://")
                    && allowed.starts_with("https://"))
                    || (origin.starts_with("http://") && allowed.starts_with("http://"));
                if schemes_match {
                    return origin.ends_with(wildcard_domain);
                }
            }
            false
        })
    }

    /// Check if method is allowed
    pub fn is_method_allowed(&self, method: &Method) -> bool {
        self.allowed_methods.iter().any(|allowed| allowed == method.as_str())
    }

    /// Check if header is allowed
    pub fn is_header_allowed(&self, header: &str) -> bool {
        self.allowed_headers
            .iter()
            .any(|allowed| allowed.to_lowercase() == header.to_lowercase() || allowed == "*")
    }
}

/// CORS middleware layer
///
/// Creates a middleware layer that handles CORS requests.
pub fn cors_layer(config: CorsConfig) -> impl tower::Layer<axum::routing::Route> + Clone {
    axum::middleware::from_fn::<_, ()>(move |request: Request, next: Next| {
        let config = config.clone();
        async move { cors_middleware(config, request, next).await }
    })
}

/// CORS middleware implementation
pub async fn cors_middleware(config: CorsConfig, request: Request, next: Next) -> Response {
    // If CORS is disabled, just pass through
    if !config.enabled {
        return next.run(request).await;
    }

    let origin =
        request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let method = request.method().clone();
    let request_headers = request.headers().keys().map(|h| h.as_str()).collect::<Vec<_>>();

    // Handle preflight requests
    if method == Method::OPTIONS {
        return handle_preflight(&config, origin, &request_headers);
    }

    let mut response = next.run(request).await;

    // Add CORS headers to regular responses
    if let Some(origin) = origin
        && config.is_origin_allowed(&origin)
    {
        if let Ok(header_value) = HeaderValue::from_str(&origin) {
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header_value);
        }

        if config.allow_credentials {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
        }

        // Add exposed headers
        if !config.exposed_headers.is_empty() {
            let exposed_headers = config.exposed_headers.join(", ");
            if let Ok(header_value) = HeaderValue::from_str(&exposed_headers) {
                response.headers_mut().insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, header_value);
            }
        }
    }

    response
}

/// Handle CORS preflight requests
fn handle_preflight(
    config: &CorsConfig,
    origin: Option<String>,
    request_headers: &[&str],
) -> Response {
    let Some(origin) = origin else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::empty())
            .unwrap();
    };

    if !config.is_origin_allowed(&origin) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(axum::body::Body::empty())
            .unwrap();
    }

    let mut response = Response::new(axum::body::Body::empty());

    // Set allowed origin
    if let Ok(header_value) = HeaderValue::from_str(&origin) {
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header_value);
    }

    // Set allowed methods
    let allowed_methods = config.allowed_methods.join(", ");
    if let Ok(header_value) = HeaderValue::from_str(&allowed_methods) {
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_METHODS, header_value);
    }

    // Set allowed headers
    let mut allowed_headers = Vec::new();
    for header in request_headers {
        if config.is_header_allowed(header) {
            allowed_headers.push(header.to_lowercase());
        }
    }

    if !allowed_headers.is_empty() {
        let headers_str = allowed_headers.join(", ");
        if let Ok(header_value) = HeaderValue::from_str(&headers_str) {
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, header_value);
        }
    }

    // Set credentials
    if config.allow_credentials {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    }

    // Set max age
    if let Some(max_age) = config.max_age
        && let Ok(header_value) = HeaderValue::from_str(&max_age.to_string())
    {
        response.headers_mut().insert(header::ACCESS_CONTROL_MAX_AGE, header_value);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(config.enabled);
        assert!(config.is_origin_allowed("http://localhost:3000"));
        assert!(config.is_method_allowed(&Method::GET));
        assert!(config.is_header_allowed("authorization"));
        assert!(!config.allow_credentials);
    }

    #[test]
    fn test_cors_config_wildcard_origin() {
        let config = CorsConfig { allowed_origins: vec!["*".to_string()], ..Default::default() };
        assert!(config.is_origin_allowed("https://any-domain.com"));
    }

    #[test]
    fn test_cors_config_subdomain_wildcard() {
        let config = CorsConfig {
            allowed_origins: vec!["https://*.example.com".to_string()],
            ..Default::default()
        };
        assert!(config.is_origin_allowed("https://api.example.com"));
        assert!(config.is_origin_allowed("https://app.example.com"));
        assert!(!config.is_origin_allowed("https://malicious.com"));
    }

    #[test]
    fn test_development_config() {
        let config = CorsConfig::development();
        assert!(config.enabled);
        assert!(config.is_origin_allowed("http://localhost:3000"));
        assert!(config.is_origin_allowed("http://localhost:8080"));
    }

    #[test]
    fn test_production_config() {
        let config = CorsConfig::production(vec!["https://example.com".to_string()]);
        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("http://localhost:3000"));
        assert!(!config.allow_credentials);
    }
}
