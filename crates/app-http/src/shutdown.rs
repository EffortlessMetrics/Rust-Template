//! Backward-compatible shutdown signal wrapper.
//!
//! The canonical shutdown signal implementation lives in `http-core`.
//! This module remains to preserve older imports like `app_http::shutdown::shutdown_signal`.

/// Re-export of [`http_core::shutdown_signal`] for backward compatibility.
pub use http_core::shutdown_signal;
