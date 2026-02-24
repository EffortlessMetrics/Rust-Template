//! Backward-compatible re-exports for platform auth.
//!
//! Canonical implementation now lives in the `http-auth` microcrate.

pub use http_auth::{Claims, PlatformAuthConfig, PlatformAuthMode, try_from_sources};
