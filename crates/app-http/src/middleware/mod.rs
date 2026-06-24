//! HTTP middleware for cross-cutting concerns
//!
//! This module contains middleware for:
//! - Request ID correlation (distributed tracing)
//! - CORS (Cross-Origin Resource Sharing) protection
//! - Security headers (CSP, XSS protection, clickjacking prevention)
//! - Platform authentication
//! - Future: Rate limiting, etc.

pub mod cors;
pub mod platform_auth;
pub mod request_id;
pub mod security_headers;

pub use cors::{CorsConfig, cors_middleware};
pub use platform_auth::{PLATFORM_AUTH_HEADER, platform_auth_guard};
pub use request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
pub use security_headers::{SecurityHeadersConfig, security_headers_middleware, CachedSecurityHeaders};
