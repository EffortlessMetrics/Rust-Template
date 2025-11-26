use spec_runtime::ValidatedConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformAuthMode {
    Open,
    Basic,
}

#[derive(Clone, Debug)]
pub struct PlatformAuthConfig {
    pub mode: PlatformAuthMode,
    pub token: Option<String>,
}

impl PlatformAuthConfig {
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        let mode_raw = std::env::var("PLATFORM_AUTH_MODE")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("platform.auth_mode"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "open".to_string());

        let token = std::env::var("PLATFORM_AUTH_TOKEN")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.auth_token").cloned()));

        Self { mode: PlatformAuthMode::from(mode_raw.as_str()), token }
    }

    pub fn requires_auth(&self) -> bool {
        matches!(self.mode, PlatformAuthMode::Basic)
    }

    pub fn is_authorized(&self, provided: Option<&str>) -> bool {
        if !self.requires_auth() {
            return true;
        }

        match (self.token.as_deref(), provided) {
            (Some(expected), Some(candidate)) => constant_time_eq(expected, candidate),
            _ => false,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            PlatformAuthMode::Open => "open",
            PlatformAuthMode::Basic => "basic",
        }
    }

    pub fn token_present(&self) -> bool {
        self.token.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    /// Emit a warning when `basic` auth is enabled without a token.
    ///
    /// Returns `true` when a warning was emitted so tests can assert the behavior without
    /// scraping logs.
    pub fn warn_if_misconfigured(&self) -> bool {
        let misconfigured = self.requires_auth() && !self.token_present();
        if misconfigured {
            tracing::warn!(
                "Platform auth is set to basic but no token was provided; writes will be rejected"
            );
        }
        misconfigured
    }
}

impl From<&str> for PlatformAuthMode {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "basic" => PlatformAuthMode::Basic,
            "none" => PlatformAuthMode::Open,
            _ => PlatformAuthMode::Open,
        }
    }
}

// Simple constant-time comparison to avoid leaking length/case differences in tokens.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auth_mode_and_warns_on_missing_token() {
        let config = PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None };
        assert!(config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(!config.is_authorized(Some("anything")));
    }

    #[test]
    fn accepts_correct_token_in_basic_mode() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: Some("secret".into()) };

        assert!(!config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(config.is_authorized(Some("secret")));
        assert!(!config.is_authorized(Some("other")));
    }

    #[test]
    fn open_mode_requires_no_token() {
        let config = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None };
        assert!(!config.requires_auth());
        assert!(config.is_authorized(None));
        assert!(config.is_authorized(Some("anything")));
    }
}
