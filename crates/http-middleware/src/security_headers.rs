//! Security headers middleware
//!
//! This module provides security headers to protect against common web vulnerabilities
//! including XSS, clickjacking, content type sniffing, and other attacks.
//!
//! # Design Philosophy
//!
//! Security headers should be:
//! - **Configurable**: Allow customization for different environments
//! - **Development-aware**: More permissive in dev, strict in prod
//! - **Comprehensive**: Cover common attack vectors
//! - **Well-documented**: Explain each header's purpose

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

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

/// Cached, pre-parsed security headers for zero-allocation application on the hot path
#[derive(Clone, Debug)]
pub struct CachedSecurityHeaders {
    content_security_policy: Option<HeaderValue>,
    x_frame_options: Option<HeaderValue>,
    x_content_type_options: Option<HeaderValue>,
    x_xss_protection: Option<HeaderValue>,
    strict_transport_security: Option<HeaderValue>,
    referrer_policy: Option<HeaderValue>,
    permissions_policy: Option<HeaderValue>,
    cross_origin_embedder_policy: Option<HeaderValue>,
    cross_origin_opener_policy: Option<HeaderValue>,
    cross_origin_resource_policy: Option<HeaderValue>,
    enabled: bool,
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
    /// Create development-friendly configuration
    ///
    /// Disables HSTS and uses more permissive CSP for local development.
    pub fn development() -> Self {
        Self {
            strict_transport_security: None, // No HSTS in development
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:*; style-src 'self' 'unsafe-inline' http://localhost:*; img-src 'self' data: https: http://localhost:*; font-src 'self' data:; connect-src 'self' ws://localhost:* wss://localhost:* http://localhost:*; frame-ancestors 'none';".to_string(),
            ),
            ..Default::default()
        }
    }

    /// Create production-optimized configuration
    ///
    /// Enables all security headers with strict settings.
    pub fn production() -> Self {
        Self {
            content_security_policy: Some(
                "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';".to_string(),
            ),
            ..Default::default()
        }
    }

    /// Pre-parse headers into a cached representation for zero-allocation access on the hot path
    pub fn cache(&self) -> CachedSecurityHeaders {
        if !self.enabled {
            return CachedSecurityHeaders {
                content_security_policy: None,
                x_frame_options: None,
                x_content_type_options: None,
                x_xss_protection: None,
                strict_transport_security: None,
                referrer_policy: None,
                permissions_policy: None,
                cross_origin_embedder_policy: None,
                cross_origin_opener_policy: None,
                cross_origin_resource_policy: None,
                enabled: false,
            };
        }

        CachedSecurityHeaders {
            content_security_policy: self
                .content_security_policy
                .as_ref()
                .and_then(|h| HeaderValue::from_str(h).ok()),
            x_frame_options: HeaderValue::from_str(&self.x_frame_options).ok(),
            x_content_type_options: HeaderValue::from_str(&self.x_content_type_options).ok(),
            x_xss_protection: HeaderValue::from_str(&self.x_xss_protection).ok(),
            strict_transport_security: self
                .strict_transport_security
                .as_ref()
                .and_then(|h| HeaderValue::from_str(h).ok()),
            referrer_policy: HeaderValue::from_str(&self.referrer_policy).ok(),
            permissions_policy: self
                .permissions_policy
                .as_ref()
                .and_then(|h| HeaderValue::from_str(h).ok()),
            cross_origin_embedder_policy: self
                .cross_origin_embedder_policy
                .as_ref()
                .and_then(|h| HeaderValue::from_str(h).ok()),
            cross_origin_opener_policy: self
                .cross_origin_opener_policy
                .as_ref()
                .and_then(|h| HeaderValue::from_str(h).ok()),
            cross_origin_resource_policy: HeaderValue::from_str(&self.cross_origin_resource_policy)
                .ok(),
            enabled: true,
        }
    }
}

impl CachedSecurityHeaders {
    /// Apply security headers to a response using cloned Bytes for zero allocation
    pub fn apply_headers(&self, response: &mut Response) {
        if !self.enabled {
            return;
        }

        // Using strictly lowercase header names prevents runtime panics in http::HeaderMap

        // Content Security Policy
        if let Some(header_value) = &self.content_security_policy {
            response.headers_mut().insert("content-security-policy", header_value.clone());
        }

        // X-Frame-Options (prevent clickjacking)
        if let Some(header_value) = &self.x_frame_options {
            response.headers_mut().insert("x-frame-options", header_value.clone());
        }

        // X-Content-Type-Options (prevent MIME sniffing)
        if let Some(header_value) = &self.x_content_type_options {
            response.headers_mut().insert("x-content-type-options", header_value.clone());
        }

        // X-XSS-Protection (legacy XSS protection)
        if let Some(header_value) = &self.x_xss_protection {
            response.headers_mut().insert("x-xss-protection", header_value.clone());
        }

        // Strict-Transport-Security (HSTS)
        if let Some(header_value) = &self.strict_transport_security {
            response.headers_mut().insert("strict-transport-security", header_value.clone());
        }

        // Referrer-Policy
        if let Some(header_value) = &self.referrer_policy {
            response.headers_mut().insert("referrer-policy", header_value.clone());
        }

        // Permissions-Policy
        if let Some(header_value) = &self.permissions_policy {
            response.headers_mut().insert("permissions-policy", header_value.clone());
        }

        // Cross-Origin-Embedder-Policy
        if let Some(header_value) = &self.cross_origin_embedder_policy {
            response.headers_mut().insert("cross-origin-embedder-policy", header_value.clone());
        }

        // Cross-Origin-Opener-Policy
        if let Some(header_value) = &self.cross_origin_opener_policy {
            response.headers_mut().insert("cross-origin-opener-policy", header_value.clone());
        }

        // Cross-Origin-Resource-Policy
        if let Some(header_value) = &self.cross_origin_resource_policy {
            response.headers_mut().insert("cross-origin-resource-policy", header_value.clone());
        }
    }
}

/// Security headers middleware layer
///
/// Creates a middleware layer that applies security headers to all responses.
pub fn security_headers_layer(
    config: SecurityHeadersConfig,
) -> impl tower::Layer<axum::routing::Route> + Clone {
    let cached_config = config.cache();
    axum::middleware::from_fn::<_, ()>(move |request: Request, next: Next| {
        let cached = cached_config.clone();
        async move {
            let mut response = next.run(request).await;
            cached.apply_headers(&mut response);
            response
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, response::Response};

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
        let cached = config.cache();
        let mut response = Response::new(Body::empty());

        cached.apply_headers(&mut response);

        let headers = response.headers();

        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("Referrer-Policy"));
    }

    #[test]
    fn test_security_headers_disabled() {
        let config = SecurityHeadersConfig { enabled: false, ..Default::default() };
        let cached = config.cache();

        let mut response = Response::new(Body::empty());
        cached.apply_headers(&mut response);

        let headers = response.headers();
        assert!(!headers.contains_key("X-Frame-Options"));
        assert!(!headers.contains_key("X-Content-Type-Options"));
    }

    #[test]
    fn test_development_config() {
        let config = SecurityHeadersConfig::development();

        // HSTS should be disabled in development
        assert!(config.strict_transport_security.is_none());

        // CSP should be more permissive
        let csp = config.content_security_policy.unwrap();
        assert!(csp.contains("'unsafe-inline'"));
        assert!(csp.contains("'unsafe-eval'"));
        assert!(csp.contains("localhost"));
    }

    #[test]
    fn test_production_config() {
        let config = SecurityHeadersConfig::production();

        // HSTS should be enabled in production
        assert!(config.strict_transport_security.is_some());

        // CSP should be stricter
        let csp = config.content_security_policy.unwrap();
        assert!(!csp.contains("'unsafe-inline'"));
        assert!(!csp.contains("'unsafe-eval'"));
        assert!(!csp.contains("localhost"));
    }
}
