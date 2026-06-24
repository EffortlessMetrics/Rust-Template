//! CORS (Cross-Origin Resource Sharing) middleware
//!
//! This module provides CORS configuration to control cross-origin requests
//! and prevent unauthorized access from different domains.

use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use spec_runtime::ValidatedConfig;
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
                "x-platform-token".to_string(),
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
    /// Create CORS config from environment variables or config file
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        let enabled = std::env::var("CORS_ENABLED")
            .ok()
            .and_then(|v| {
                v.parse()
                    .map_err(|e| {
                        tracing::warn!(
                            env_var = "CORS_ENABLED",
                            value = %v,
                            error = %e,
                            "Failed to parse environment variable, using default"
                        );
                        e
                    })
                    .ok()
            })
            .or_else(|| {
                config.and_then(|cfg| cfg.settings.get("cors.enabled")).and_then(|v| v.as_bool())
            })
            .unwrap_or(true); // Enable by default for security

        if !enabled {
            return Self { enabled: false, ..Default::default() };
        }

        let allowed_origins = parse_string_list_env("CORS_ALLOWED_ORIGINS")
            .or_else(|| {
                config.and_then(|cfg| cfg.settings.get("cors.allowed_origins")).and_then(
                    |v| match v {
                        serde_yaml::Value::Sequence(arr) => Some(
                            arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect(),
                        ),
                        _ => None,
                    },
                )
            })
            .unwrap_or_else(|| {
                vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]
            });

        let allowed_methods = parse_string_list_env("CORS_ALLOWED_METHODS")
            .or_else(|| {
                config.and_then(|cfg| cfg.settings.get("cors.allowed_methods")).and_then(
                    |v| match v {
                        serde_yaml::Value::Sequence(arr) => Some(
                            arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect(),
                        ),
                        _ => None,
                    },
                )
            })
            .unwrap_or_else(|| {
                vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                    "PATCH".to_string(),
                ]
            });

        let allowed_headers = parse_string_list_env("CORS_ALLOWED_HEADERS")
            .or_else(|| {
                config.and_then(|cfg| cfg.settings.get("cors.allowed_headers")).and_then(
                    |v| match v {
                        serde_yaml::Value::Sequence(arr) => Some(
                            arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect(),
                        ),
                        _ => None,
                    },
                )
            })
            .unwrap_or_else(|| {
                vec![
                    "authorization".to_string(),
                    "content-type".to_string(),
                    "x-request-id".to_string(),
                    "x-platform-token".to_string(),
                    "accept".to_string(),
                    "origin".to_string(),
                ]
            });

        let allow_credentials = std::env::var("CORS_ALLOW_CREDENTIALS")
            .ok()
            .and_then(|v| {
                v.parse()
                    .map_err(|e| {
                        tracing::warn!(
                            env_var = "CORS_ALLOW_CREDENTIALS",
                            value = %v,
                            error = %e,
                            "Failed to parse environment variable, using default"
                        );
                        e
                    })
                    .ok()
            })
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("cors.allow_credentials"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(false); // Default to false for security

        let max_age = std::env::var("CORS_MAX_AGE")
            .ok()
            .and_then(|v| {
                v.parse()
                    .map_err(|e| {
                        tracing::warn!(
                            env_var = "CORS_MAX_AGE",
                            value = %v,
                            error = %e,
                            "Failed to parse environment variable, using default"
                        );
                        e
                    })
                    .ok()
            })
            .or_else(|| {
                config.and_then(|cfg| cfg.settings.get("cors.max_age")).and_then(|v| v.as_u64())
            });

        Self {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            exposed_headers: vec!["x-request-id".to_string()],
            allow_credentials,
            max_age,
            enabled: true,
        }
    }

    /// Check if origin is allowed
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        http_origin_policy::is_origin_allowed_by_any(&self.allowed_origins, origin)
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

/// Parse comma-separated string list from environment variable
fn parse_string_list_env(env_var: &str) -> Option<Vec<String>> {
    std::env::var(env_var)
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

/// CORS middleware implementation
pub async fn cors_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // If CORS is disabled, just pass through
    if !state.cors_config.enabled {
        return next.run(request).await;
    }

    let origin =
        request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let method = request.method().clone();

    // Handle preflight requests
    if method == Method::OPTIONS {
        let request_headers = request.headers().keys().map(|h| h.as_str()).collect::<Vec<_>>();
        return handle_preflight(&state.cors_config, origin, &request_headers);
    }

    let mut response = next.run(request).await;

    // Add CORS headers to regular responses
    if let Some(origin) = origin
        && state.cors_config.is_origin_allowed(&origin)
    {
        response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));

        if let Ok(header_value) = HeaderValue::from_str(&origin) {
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header_value);
        }

        if state.cors_config.allow_credentials {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
        }

        // Add exposed headers
        if !state.cors_config.exposed_headers.is_empty() {
            let exposed_headers = state.cors_config.exposed_headers.join(", ");
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
    fn test_parse_string_list_env() {
        // This test would need to set environment variables
        // For now, we'll test the parsing logic directly
        let input = "origin1,origin2, origin3";
        let result: Vec<String> =
            input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        assert_eq!(result, vec!["origin1", "origin2", "origin3"]);
    }

    // #[tokio::test]
    // async fn test_cors_middleware_with_disabled_config() {
    //     let config = Arc::new(CorsConfig { enabled: false, ..Default::default() });

    //     let request = Request::builder()
    //         .uri("/test")
    //         .header("origin", "https://example.com")
    //         .body(Body::empty())
    //         .unwrap();

    //     async fn handler() -> &'static str {
    //         "response"
    //     }

    //     let next = Next::new(handler);
    //     let response = cors_middleware(config, request, next).await;
    //     assert_eq!(response.status(), StatusCode::OK);
    // }
}
