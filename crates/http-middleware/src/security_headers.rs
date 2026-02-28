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
use spec_runtime::ValidatedConfig;

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
    /// Create security headers config from environment variables or validated config.
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        let enabled = std::env::var("SECURITY_HEADERS_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("security_headers.enabled"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true);

        if !enabled {
            return Self { enabled: false, ..Default::default() };
        }

        let is_development =
            std::env::var("ENV").unwrap_or_else(|_| "development".to_string()).to_lowercase()
                == "development";

        let content_security_policy = std::env::var("CSP_HEADER")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.content_security_policy"))
            .or_else(|| {
                if is_development {
                    Some("default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:*; style-src 'self' 'unsafe-inline' http://localhost:*; img-src 'self' data: https: http://localhost:*; font-src 'self' data:; connect-src 'self' ws://localhost:* wss://localhost:* http://localhost:*; frame-ancestors 'none';".to_string())
                } else {
                    Some("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';".to_string())
                }
            });

        let x_frame_options = std::env::var("X_FRAME_OPTIONS")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.x_frame_options"))
            .unwrap_or_else(|| "DENY".to_string());

        let x_content_type_options = std::env::var("X_CONTENT_TYPE_OPTIONS")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.x_content_type_options"))
            .unwrap_or_else(|| "nosniff".to_string());

        let x_xss_protection = std::env::var("X_XSS_PROTECTION")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.x_xss_protection"))
            .unwrap_or_else(|| "1; mode=block".to_string());

        let strict_transport_security = if is_development {
            None
        } else {
            std::env::var("STRICT_TRANSPORT_SECURITY")
                .ok()
                .or_else(|| {
                    get_setting_string(config, "security_headers.strict_transport_security")
                })
                .or_else(|| Some("max-age=31536000; includeSubDomains; preload".to_string()))
        };

        let referrer_policy = std::env::var("REFERRER_POLICY")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.referrer_policy"))
            .unwrap_or_else(|| "strict-origin-when-cross-origin".to_string());

        let permissions_policy = std::env::var("PERMISSIONS_POLICY")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.permissions_policy"))
            .or_else(|| Some("geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=(), autoplay=(), encrypted-media=(), fullscreen=(), picture-in-picture=()".to_string()));

        let cross_origin_embedder_policy = std::env::var("CROSS_ORIGIN_EMBEDDER_POLICY")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.cross_origin_embedder_policy"))
            .or_else(|| Some("require-corp".to_string()));

        let cross_origin_opener_policy = std::env::var("CROSS_ORIGIN_OPENER_POLICY")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.cross_origin_opener_policy"))
            .or_else(|| Some("same-origin".to_string()));

        let cross_origin_resource_policy = std::env::var("CROSS_ORIGIN_RESOURCE_POLICY")
            .ok()
            .or_else(|| get_setting_string(config, "security_headers.cross_origin_resource_policy"))
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

    /// Apply security headers to a response
    pub fn apply_headers(&self, response: &mut Response) {
        if !self.enabled {
            return;
        }

        // Content Security Policy
        if let Some(csp) = &self.content_security_policy
            && let Ok(header_value) = HeaderValue::from_str(csp)
        {
            response.headers_mut().insert("Content-Security-Policy", header_value);
        }

        // X-Frame-Options (prevent clickjacking)
        if let Ok(header_value) = HeaderValue::from_str(&self.x_frame_options) {
            response.headers_mut().insert("X-Frame-Options", header_value);
        }

        // X-Content-Type-Options (prevent MIME sniffing)
        if let Ok(header_value) = HeaderValue::from_str(&self.x_content_type_options) {
            response.headers_mut().insert("X-Content-Type-Options", header_value);
        }

        // X-XSS-Protection (legacy XSS protection)
        if let Ok(header_value) = HeaderValue::from_str(&self.x_xss_protection) {
            response.headers_mut().insert("X-XSS-Protection", header_value);
        }

        // Strict-Transport-Security (HSTS)
        if let Some(sts) = &self.strict_transport_security
            && let Ok(header_value) = HeaderValue::from_str(sts)
        {
            response.headers_mut().insert("Strict-Transport-Security", header_value);
        }

        // Referrer-Policy
        if let Ok(header_value) = HeaderValue::from_str(&self.referrer_policy) {
            response.headers_mut().insert("Referrer-Policy", header_value);
        }

        // Permissions-Policy
        if let Some(pp) = &self.permissions_policy
            && let Ok(header_value) = HeaderValue::from_str(pp)
        {
            response.headers_mut().insert("Permissions-Policy", header_value);
        }

        // Cross-Origin-Embedder-Policy
        if let Some(coep) = &self.cross_origin_embedder_policy
            && let Ok(header_value) = HeaderValue::from_str(coep)
        {
            response.headers_mut().insert("Cross-Origin-Embedder-Policy", header_value);
        }

        // Cross-Origin-Opener-Policy
        if let Some(coop) = &self.cross_origin_opener_policy
            && let Ok(header_value) = HeaderValue::from_str(coop)
        {
            response.headers_mut().insert("Cross-Origin-Opener-Policy", header_value);
        }

        // Cross-Origin-Resource-Policy
        if let Ok(header_value) = HeaderValue::from_str(&self.cross_origin_resource_policy) {
            response.headers_mut().insert("Cross-Origin-Resource-Policy", header_value);
        }
    }
}

fn get_setting_string(config: Option<&ValidatedConfig>, key: &str) -> Option<String> {
    config.and_then(|cfg| cfg.settings.get(key)).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Security headers middleware layer
///
/// Creates a middleware layer that applies security headers to all responses.
pub fn security_headers_layer(
    config: SecurityHeadersConfig,
) -> impl tower::Layer<axum::routing::Route> + Clone {
    axum::middleware::from_fn::<_, ()>(move |request: Request, next: Next| {
        let config = config.clone();
        async move {
            let mut response = next.run(request).await;
            config.apply_headers(&mut response);
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
        let config = SecurityHeadersConfig { enabled: false, ..Default::default() };

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
