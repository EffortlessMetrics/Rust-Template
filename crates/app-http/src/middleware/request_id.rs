//! Backward-compatible request-id exports.
//!
//! Canonical implementation now lives in the `http-middleware` microcrate.

pub use http_middleware::request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
