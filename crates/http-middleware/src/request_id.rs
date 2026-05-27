//! Backward-compatible request ID exports.
//!
//! The request ID middleware implementation lives in the dedicated
//! `http-request-id` microcrate to keep middleware concerns SRP-focused.

pub use http_request_id::{REQUEST_ID_HEADER, RequestId, request_id_layer, request_id_middleware};
