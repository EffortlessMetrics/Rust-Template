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

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
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

    /// Pre-parse configuration into cached headers
    pub fn cache(&self) -> CachedSecurityHeaders {
        let mut headers = Vec::new();

        if !self.enabled {
            return CachedSecurityHeaders { enabled: false, headers };
        }

        // Content Security Policy
        if let Some(csp) = &self.content_security_policy
            && let Ok(val) = HeaderValue::from_str(csp)
        {
            headers.push((HeaderName::from_static("content-security-policy"), val));
        }

        // X-Frame-Options
        if let Ok(val) = HeaderValue::from_str(&self.x_frame_options) {
            headers.push((HeaderName::from_static("x-frame-options"), val));
        }

        // X-Content-Type-Options
        if let Ok(val) = HeaderValue::from_str(&self.x_content_type_options) {
            headers.push((HeaderName::from_static("x-content-type-options"), val));
        }

        // X-XSS-Protection
        if let Ok(val) = HeaderValue::from_str(&self.x_xss_protection) {
            headers.push((HeaderName::from_static("x-xss-protection"), val));
        }

        // Strict-Transport-Security
        if let Some(sts) = &self.strict_transport_security
            && let Ok(val) = HeaderValue::from_str(sts)
        {
            headers.push((HeaderName::from_static("strict-transport-security"), val));
        }

        // Referrer-Policy
        if let Ok(val) = HeaderValue::from_str(&self.referrer_policy) {
            headers.push((HeaderName::from_static("referrer-policy"), val));
        }

        // Permissions-Policy
        if let Some(pp) = &self.permissions_policy
            && let Ok(val) = HeaderValue::from_str(pp)
        {
            headers.push((HeaderName::from_static("permissions-policy"), val));
        }

        // Cross-Origin-Embedder-Policy
        if let Some(coep) = &self.cross_origin_embedder_policy
            && let Ok(val) = HeaderValue::from_str(coep)
        {
            headers.push((HeaderName::from_static("cross-origin-embedder-policy"), val));
        }

        // Cross-Origin-Opener-Policy
        if let Some(coop) = &self.cross_origin_opener_policy
            && let Ok(val) = HeaderValue::from_str(coop)
        {
            headers.push((HeaderName::from_static("cross-origin-opener-policy"), val));
        }

        // Cross-Origin-Resource-Policy
        if let Ok(val) = HeaderValue::from_str(&self.cross_origin_resource_policy) {
            headers.push((HeaderName::from_static("cross-origin-resource-policy"), val));
        }

        CachedSecurityHeaders { enabled: self.enabled, headers }
    }
}

/// Pre-parsed security headers ready for zero-allocation cloning on the hot path
#[derive(Clone, Debug)]
pub struct CachedSecurityHeaders {
    pub enabled: bool,
    pub headers: Vec<(HeaderName, HeaderValue)>,
}

impl Default for CachedSecurityHeaders {
    fn default() -> Self {
        SecurityHeadersConfig::default().cache()
    }
}

impl CachedSecurityHeaders {
    /// Apply pre-parsed security headers to a response
    pub fn apply_headers(&self, response: &mut Response) {
        if !self.enabled {
            return;
        }

        for (name, value) in &self.headers {
            response.headers_mut().insert(name.clone(), value.clone());
        }
    }
}

/// Security headers middleware layer
///
/// Creates a middleware layer that applies security headers to all responses.
pub fn security_headers_layer(
    config: SecurityHeadersConfig,
) -> impl tower::Layer<axum::routing::Route> + Clone {
    let cached = config.cache();
    axum::middleware::from_fn::<_, ()>(move |request: Request, next: Next| {
        let cached = cached.clone();
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
        let config = SecurityHeadersConfig::default().cache();
        let mut response = Response::new(Body::empty());

        config.apply_headers(&mut response);

        let headers = response.headers();

        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("Referrer-Policy"));
    }

    #[test]
    fn test_security_headers_disabled() {
        let config = SecurityHeadersConfig { enabled: false, ..Default::default() }.cache();

        let mut response = Response::new(Body::empty());
        config.apply_headers(&mut response);

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
