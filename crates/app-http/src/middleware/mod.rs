//! HTTP middleware for cross-cutting concerns
//!
//! This module contains middleware for:
//! - Request ID correlation (distributed tracing)
//! - Future: Rate limiting, authentication, etc.

pub mod platform_auth;
pub mod request_id;

pub use platform_auth::{PLATFORM_AUTH_HEADER, platform_auth_guard};
pub use request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
