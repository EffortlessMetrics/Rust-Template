//! Platform HTTP authentication primitives.
//!
//! This crate is a compatibility adapter over `http_auth_config_loader`.

pub use http_auth_config_loader::{Claims, PlatformAuthConfig, PlatformAuthMode, try_from_sources};
