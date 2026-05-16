use crate::{PlatformAuthConfig, PlatformAuthMode};
use http_auth_verifier::authorize_token;

impl PlatformAuthConfig {
    /// True when mode requires auth checks (`basic` or `jwt`).
    pub fn requires_auth(&self) -> bool {
        matches!(self.mode, PlatformAuthMode::Basic | PlatformAuthMode::Jwt)
    }

    /// True when mode requires auth and matching credential is configured.
    pub fn can_enforce_auth(&self) -> bool {
        match self.mode {
            PlatformAuthMode::Open => false,
            PlatformAuthMode::Basic => self.has_basic_token(),
            PlatformAuthMode::Jwt => self.has_jwt_secret(),
        }
    }

    /// Validate a provided token according to current mode.
    pub fn is_authorized(&self, provided: Option<&str>) -> bool {
        if !self.requires_auth() {
            return true;
        }

        authorize_token(provided, self.token.as_deref(), self.jwt_secret.as_deref())
    }

    /// Lowercase label for UI/status responses.
    pub fn mode_label(&self) -> &'static str {
        self.mode.label()
    }

    /// True when required credential for current mode is present.
    pub fn token_present(&self) -> bool {
        match self.mode {
            PlatformAuthMode::Basic => self.has_basic_token(),
            PlatformAuthMode::Jwt => self.has_jwt_secret(),
            PlatformAuthMode::Open => true,
        }
    }

    /// Warn when auth mode is enabled but no credentials exist.
    ///
    /// Returns true when warning condition is hit (for deterministic tests).
    pub fn warn_if_misconfigured(&self) -> bool {
        let misconfigured = self.requires_auth() && !self.has_any_credential();

        if misconfigured {
            match self.mode {
                PlatformAuthMode::Basic | PlatformAuthMode::Jwt => {
                    tracing::warn!(
                        "Platform auth is enabled but no PLATFORM_AUTH_TOKEN or PLATFORM_JWT_SECRET was provided; writes will be rejected"
                    );
                }
                PlatformAuthMode::Open => {}
            }
        }
        misconfigured
    }

    fn has_any_credential(&self) -> bool {
        self.has_basic_token() || self.has_jwt_secret()
    }

    fn has_basic_token(&self) -> bool {
        self.token.as_ref().is_some_and(|t| !t.is_empty())
    }

    fn has_jwt_secret(&self) -> bool {
        self.jwt_secret.as_ref().is_some_and(|s| !s.is_empty())
    }
}
