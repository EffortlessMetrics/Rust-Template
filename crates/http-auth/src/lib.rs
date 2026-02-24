//! Backward-compatibility auth primitives for HTTP platform auth.
//!
//! This crate now exposes the auth-policy model through `http-auth-config`.

pub use http_auth_config::{Claims, PlatformAuthConfig, PlatformAuthMode, try_from_sources};
