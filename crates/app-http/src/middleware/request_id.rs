//! Backward-compatible request ID middleware exports.
//!
//! Canonical implementation lives in the `http-middleware` microcrate.

pub use http_middleware::{REQUEST_ID_HEADER, RequestId, request_id_middleware};
