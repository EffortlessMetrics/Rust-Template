//! Shared HTTP middleware for cross-cutting concerns
//!
//! This crate provides reusable middleware for:
//! - Request ID correlation (distributed tracing)
//! - CORS (Cross-Origin Resource Sharing) protection
//! - Security headers (CSP, XSS protection, clickjacking prevention)
//!
//! # Design Philosophy
//!
//! Middleware should be:
//! - **Composable**: Easy to combine and layer
//! - **Configurable**: Clear configuration structs
//! - **Framework-agnostic**: Where possible, avoid framework-specific state
//! - **Observable**: Integrate with tracing

pub mod cors;
pub mod request_id;
pub mod security_headers;

pub use cors::{CorsConfig, cors_layer};
pub use request_id::{REQUEST_ID_HEADER, RequestId, request_id_layer};
pub use security_headers::{SecurityHeadersConfig, security_headers_layer};
