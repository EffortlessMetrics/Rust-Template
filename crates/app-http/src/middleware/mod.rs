//! HTTP middleware for cross-cutting concerns
//!
//! This module contains middleware for:
//! - Request ID correlation (distributed tracing)
//! - Future: Rate limiting, authentication, etc.

pub mod request_id;

pub use request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
