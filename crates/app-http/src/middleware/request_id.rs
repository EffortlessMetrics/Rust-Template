//! Backward-compatible request ID exports.
//!
//! Request ID middleware now lives in `http-request-id` and is re-exported here
//! to avoid breaking existing `app_http::middleware::request_id::*` imports.

pub use http_request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
