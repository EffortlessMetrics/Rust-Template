//! Platform HTTP authentication primitives.
//!
//! This crate owns credential sourcing and auth-mode policy decisions.
//! `PlatformAuthMode` parsing lives in `http-auth-mode` and is re-exported here
//! for compatibility.

mod config;
mod policy;
mod source;

#[cfg(test)]
mod test_support;

pub use config::PlatformAuthConfig;
pub use http_auth_mode::PlatformAuthMode;
pub use http_auth_verifier::Claims;

#[cfg(test)]
mod tests;
