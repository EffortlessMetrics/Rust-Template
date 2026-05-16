use crate::PlatformAuthMode;

/// Runtime auth configuration used by HTTP middleware.
#[derive(Clone, Debug)]
pub struct PlatformAuthConfig {
    /// Auth mode to enforce.
    pub mode: PlatformAuthMode,
    /// Shared basic token (for `Basic` mode).
    pub token: Option<String>,
    /// JWT secret key (for `Jwt` mode).
    pub jwt_secret: Option<String>,
}
