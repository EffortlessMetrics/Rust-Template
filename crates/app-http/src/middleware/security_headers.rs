//! Security headers middleware
//!
//! This module provides security headers to protect against common web vulnerabilities
//! including XSS, clickjacking, content type sniffing, and other attacks.

use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use spec_runtime::ValidatedConfig;

use crate::AppState;

/// Security headers configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SecurityHeadersConfig {
    /// Content Security Policy (CSP) header
    pub content_security_policy: Option<String>,
    /// X-Frame-Options header to prevent clickjacking
    pub x_frame_options: String,
    /// X-Content-Type-Options header to prevent MIME type sniffing
    pub x_content_type_options: String,
    /// X-XSS-Protection header for XSS filtering
    pub x_xss_protection: String,
    /// Strict-Transport-Security header for HTTPS enforcement
    pub strict_transport_security: Option<String>,
    /// Referrer-Policy header
    pub referrer_policy: String,
    /// Permissions-Policy header
    pub permissions_policy: Option<String>,
    /// Cross-Origin-Embedder-Policy header
    pub cross_origin_embedder_policy: Option<String>,
    /// Cross-Origin-Opener-Policy header
    pub cross_origin_opener_policy: Option<String>,
    /// Cross-Origin-Resource-Policy header
    pub cross_origin_resource_policy: String,
    /// Whether security headers are enabled
    pub enabled: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            // Strict CSP for production, more permissive for development
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';".to_string(),
            ),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: "nosniff".to_string(),
            x_xss_protection: "1; mode=block".to_string(),
            // HSTS with 1 year max age, include subdomains, preload
            strict_transport_security: Some("max-age=31536000; includeSubDomains; preload".to_string()),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            // Restrict various features for security
            permissions_policy: Some(
                "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=(), autoplay=(), encrypted-media=(), fullscreen=(), picture-in-picture=()".to_string(),
            ),
            cross_origin_embedder_policy: Some("require-corp".to_string()),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            cross_origin_resource_policy: "same-origin".to_string(),
            enabled: true,
        }
    }
}

impl SecurityHeadersConfig {
    /// Create security headers config from environment variables or config file
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        let enabled = std::env::var("SECURITY_HEADERS_ENABLED")
            .ok()
            .and_then(|v| {
                v.parse()
                    .map_err(|e| {
                        tracing::warn!(
                            env_var = "SECURITY_HEADERS_ENABLED",
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
                    .and_then(|cfg| cfg.settings.get("security_headers.enabled"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true); // Enable by default for security

        if !enabled {
            return Self { enabled: false, ..Default::default() };
        }

        let is_development =
            std::env::var("ENV").unwrap_or_else(|_| "development".to_string()).to_lowercase()
                == "development";

        let content_security_policy = std::env::var("CSP_HEADER")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.content_security_policy"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| {
                if is_development {
                    // More permissive CSP for development
                    Some(
                        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:*; style-src 'self' 'unsafe-inline' http://localhost:*; img-src 'self' data: https: http://localhost:*; font-src 'self' data:; connect-src 'self' ws://localhost:* wss://localhost:* http://localhost:*; frame-ancestors 'none';".to_string(),
                    )
                } else {
                    // Strict CSP for production
                    Some(
                        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';".to_string(),
                    )
                }
            });

        let x_frame_options = std::env::var("X_FRAME_OPTIONS")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.x_frame_options"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "DENY".to_string());

        let x_content_type_options = std::env::var("X_CONTENT_TYPE_OPTIONS")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.x_content_type_options"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "nosniff".to_string());

        let x_xss_protection = std::env::var("X_XSS_PROTECTION")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.x_xss_protection"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "1; mode=block".to_string());

        let strict_transport_security = if is_development {
            // Don't use HSTS in development
            None
        } else {
            std::env::var("STRICT_TRANSPORT_SECURITY")
                .ok()
                .or_else(|| {
                    config
                        .and_then(|cfg| {
                            cfg.settings.get("security_headers.strict_transport_security")
                        })
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| Some("max-age=31536000; includeSubDomains; preload".to_string()))
        };

        let referrer_policy = std::env::var("REFERRER_POLICY")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.referrer_policy"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "strict-origin-when-cross-origin".to_string());

        let permissions_policy = std::env::var("PERMISSIONS_POLICY")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.permissions_policy"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| {
                Some(
                    "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=(), autoplay=(), encrypted-media=(), fullscreen=(), picture-in-picture=()".to_string(),
                )
            });

        let cross_origin_embedder_policy = std::env::var("CROSS_ORIGIN_EMBEDDER_POLICY")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| {
                        cfg.settings.get("security_headers.cross_origin_embedder_policy")
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| Some("require-corp".to_string()));

        let cross_origin_opener_policy = std::env::var("CROSS_ORIGIN_OPENER_POLICY")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.cross_origin_opener_policy"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| Some("same-origin".to_string()));

        let cross_origin_resource_policy = std::env::var("CROSS_ORIGIN_RESOURCE_POLICY")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| {
                        cfg.settings.get("security_headers.cross_origin_resource_policy")
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "same-origin".to_string());

        Self {
            content_security_policy,
            x_frame_options,
            x_content_type_options,
            x_xss_protection,
            strict_transport_security,
            referrer_policy,
            permissions_policy,
            cross_origin_embedder_policy,
            cross_origin_opener_policy,
            cross_origin_resource_policy,
            enabled: true,
        }
    }

    /// Apply security headers to a response (legacy dynamic parsing)
    pub fn apply_headers(&self, response: &mut Response) {
        self.cache().apply_headers(response);
    }

    /// Pre-parse header values to avoid per-request allocations
    pub fn cache(&self) -> CachedSecurityHeaders {
        let mut headers = Vec::with_capacity(10);
        if let Some(csp) = &self.content_security_policy
            && let Ok(v) = HeaderValue::from_str(csp)
        {
            headers.push(("content-security-policy", v));
        }
        if let Ok(v) = HeaderValue::from_str(&self.x_frame_options) {
            headers.push(("x-frame-options", v));
        }
        if let Ok(v) = HeaderValue::from_str(&self.x_content_type_options) {
            headers.push(("x-content-type-options", v));
        }
        if let Ok(v) = HeaderValue::from_str(&self.x_xss_protection) {
            headers.push(("x-xss-protection", v));
        }
        if let Some(sts) = &self.strict_transport_security
            && let Ok(v) = HeaderValue::from_str(sts)
        {
            headers.push(("strict-transport-security", v));
        }
        if let Ok(v) = HeaderValue::from_str(&self.referrer_policy) {
            headers.push(("referrer-policy", v));
        }
        if let Some(pp) = &self.permissions_policy
            && let Ok(v) = HeaderValue::from_str(pp)
        {
            headers.push(("permissions-policy", v));
        }
        if let Some(coep) = &self.cross_origin_embedder_policy
            && let Ok(v) = HeaderValue::from_str(coep)
        {
            headers.push(("cross-origin-embedder-policy", v));
        }
        if let Some(coop) = &self.cross_origin_opener_policy
            && let Ok(v) = HeaderValue::from_str(coop)
        {
            headers.push(("cross-origin-opener-policy", v));
        }
        if let Ok(v) = HeaderValue::from_str(&self.cross_origin_resource_policy) {
            headers.push(("cross-origin-resource-policy", v));
        }

        CachedSecurityHeaders { enabled: self.enabled, headers }
    }
}

/// Pre-parsed security headers
#[derive(Clone, Debug)]
pub struct CachedSecurityHeaders {
    pub enabled: bool,
    pub headers: Vec<(&'static str, HeaderValue)>,
}

impl CachedSecurityHeaders {
    /// Apply security headers to a response
    pub fn apply_headers(&self, response: &mut Response) {
        if !self.enabled {
            return;
        }
        for (name, value) in &self.headers {
            response.headers_mut().insert(*name, value.clone());
        }
    }
}

/// Security headers middleware implementation
pub async fn security_headers_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    state.cached_security_headers.apply_headers(&mut response);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::HeaderValue, response::Response};
    use serial_test::serial;
    use testing::process::EnvVarGuard;

    #[test]
    fn test_security_headers_config_default() {
        let config = SecurityHeadersConfig::default();
        assert!(config.enabled);
        assert_eq!(config.x_frame_options, "DENY");
        assert_eq!(config.x_content_type_options, "nosniff");
        assert_eq!(config.x_xss_protection, "1; mode=block");
        assert!(config.content_security_policy.is_some());
        assert!(config.strict_transport_security.is_some());
    }

    #[test]
    fn test_security_headers_apply_headers() {
        let config = SecurityHeadersConfig::default();
        let mut response = Response::new(Body::empty());

        config.apply_headers(&mut response);

        let headers = response.headers();

        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("Referrer-Policy"));

        assert_eq!(headers.get("X-Frame-Options").unwrap(), HeaderValue::from_static("DENY"));
        assert_eq!(
            headers.get("X-Content-Type-Options").unwrap(),
            HeaderValue::from_static("nosniff")
        );
    }

    #[test]
    fn test_security_headers_disabled() {
        let config = SecurityHeadersConfig { enabled: false, ..Default::default() };

        let mut response = Response::new(Body::empty());
        config.apply_headers(&mut response);

        let headers = response.headers();
        assert!(!headers.contains_key("X-Frame-Options"));
        assert!(!headers.contains_key("X-Content-Type-Options"));
    }

    // #[tokio::test]
    // async fn test_security_headers_middleware() {
    //     let config = Arc::new(SecurityHeadersConfig::default());

    //     async fn handler() -> &'static str {
    //         "response"
    //     }

    //     let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    //     let next = Next::new(handler);
    //     let response = security_headers_middleware(config, request, next).await;

    //     assert!(response.headers().contains_key("X-Frame-Options"));
    //     assert!(response.headers().contains_key("X-Content-Type-Options"));
    //     assert!(response.headers().contains_key("X-XSS-Protection"));
    // }

    #[test]
    #[serial]
    fn test_development_csp() {
        // Test that development CSP is more permissive
        let guard = EnvVarGuard::new(&["ENV"]);
        guard.set("ENV", "development");

        let config = SecurityHeadersConfig::from_sources(None);

        let csp = config.content_security_policy.unwrap();
        assert!(csp.contains("'unsafe-inline'"));
        assert!(csp.contains("'unsafe-eval'"));
        assert!(csp.contains("localhost"));
    }

    #[test]
    #[serial]
    fn test_production_csp() {
        // Test that production CSP is stricter
        let guard = EnvVarGuard::new(&["ENV"]);
        guard.set("ENV", "production");

        let config = SecurityHeadersConfig::from_sources(None);

        let csp = config.content_security_policy.unwrap();
        assert!(!csp.contains("'unsafe-inline'"));
        assert!(!csp.contains("'unsafe-eval'"));
        assert!(!csp.contains("localhost"));
    }

    #[test]
    #[serial]
    fn test_hsts_disabled_in_development() {
        let guard = EnvVarGuard::new(&["ENV"]);
        guard.set("ENV", "development");

        let config = SecurityHeadersConfig::from_sources(None);

        assert!(config.strict_transport_security.is_none());
    }
}
