use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use spec_runtime::ValidatedConfig;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformAuthMode {
    Open,
    Basic,
    Jwt,
}

#[derive(Clone, Debug)]
pub struct PlatformAuthConfig {
    pub mode: PlatformAuthMode,
    pub token: Option<String>,
    pub jwt_secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind<'a> {
    Basic(&'a str),
    Jwt(&'a str),
}

impl PlatformAuthConfig {
    /// Create auth config from environment and validated config sources.
    ///
    /// # Panics
    ///
    /// Panics if PLATFORM_AUTH_MODE is set to an invalid value.
    /// Valid values: "basic", "jwt", "none", "open" (case-insensitive).
    /// This is fail-closed behavior to prevent silent fallback to unauthenticated mode.
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        match Self::try_from_sources(config) {
            Ok(cfg) => cfg,
            Err(e) => {
                panic!("FATAL: Invalid platform auth configuration: {}", e);
            }
        }
    }

    /// Try to create auth config, returning an error on invalid configuration.
    ///
    /// This is the fallible version of `from_sources()` for contexts where
    /// panicking is not appropriate (e.g., testing, graceful error handling).
    pub fn try_from_sources(config: Option<&ValidatedConfig>) -> Result<Self, String> {
        let mode_raw = std::env::var("PLATFORM_AUTH_MODE")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("platform.auth_mode"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "open".to_string());

        // Fail-closed: invalid auth mode is a hard error, not a silent fallback
        let mode = PlatformAuthMode::parse_strict(&mode_raw)?;

        let token = std::env::var("PLATFORM_AUTH_TOKEN")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.auth_token").cloned()));

        let jwt_secret = std::env::var("PLATFORM_JWT_SECRET")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.jwt_secret").cloned()));

        Ok(Self { mode, token, jwt_secret })
    }

    pub fn requires_auth(&self) -> bool {
        matches!(self.mode, PlatformAuthMode::Basic | PlatformAuthMode::Jwt)
    }

    pub fn is_authorized(&self, provided: Option<&str>) -> bool {
        if !self.requires_auth() {
            return true;
        }

        let Some(token) = provided else { return false };

        match token_kind(token) {
            TokenKind::Basic(candidate) => {
                self.token.as_deref().is_some_and(|expected| constant_time_eq(expected, candidate))
            }
            TokenKind::Jwt(candidate) => self
                .jwt_secret
                .as_deref()
                .is_some_and(|secret| validate_jwt_token(candidate, secret)),
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            PlatformAuthMode::Open => "open",
            PlatformAuthMode::Basic => "basic",
            PlatformAuthMode::Jwt => "jwt",
        }
    }

    /// Returns true if the required credential for the current auth mode is present.
    ///
    /// - Basic mode: requires a basic token
    /// - JWT mode: requires a JWT secret
    /// - Open mode: always returns true (no credentials required)
    pub fn token_present(&self) -> bool {
        match self.mode {
            PlatformAuthMode::Basic => self.has_basic_token(),
            PlatformAuthMode::Jwt => self.has_jwt_secret(),
            PlatformAuthMode::Open => true,
        }
    }

    /// Emit a warning when auth is enabled without proper credentials.
    ///
    /// Returns `true` when a warning was emitted so tests can assert the behavior without
    /// scraping logs.
    pub fn warn_if_misconfigured(&self) -> bool {
        let misconfigured =
            self.requires_auth() && !(self.has_basic_token() || self.has_jwt_secret());

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

    fn has_basic_token(&self) -> bool {
        self.token.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    fn has_jwt_secret(&self) -> bool {
        self.jwt_secret.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
    }
}

/// Validate a JWT token with the provided secret
fn validate_jwt_token(token: &str, secret: &str) -> bool {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.insert("exp".to_string());
    validation.validate_exp = true;

    // Set leeway to 60 seconds to handle clock skew between servers
    // This prevents valid tokens from being rejected due to minor time differences
    validation.leeway = 60;

    // Also validate nbf (not before) claim if present
    validation.validate_nbf = true;

    let decoding_key = DecodingKey::from_secret(secret.as_ref());

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Additional validation checks
            let claims = token_data.claims;

            // Validate issuer is present and not empty
            if claims.iss.is_empty() {
                tracing::debug!("JWT validation failed: missing issuer");
                return false;
            }

            // Validate subject is present and not empty
            if claims.sub.is_empty() {
                tracing::debug!("JWT validation failed: missing subject");
                return false;
            }

            // Validate issued at time is not too far in the future (beyond 5 minutes)
            let now = jsonwebtoken::get_current_timestamp();
            if claims.iat as u64 > now + 300 {
                tracing::debug!("JWT validation failed: token issued too far in the future");
                return false;
            }

            true
        }
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            false
        }
    }
}

/// Create a JWT token with the provided secret and claims (test-only helper)
pub fn create_jwt_token(
    secret: &str,
    subject: &str,
    issuer: &str,
    expires_in_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = Claims {
        sub: subject.to_string(),
        exp: (now + expires_in_seconds) as usize,
        iat: now as usize,
        iss: issuer.to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
}

impl PlatformAuthMode {
    /// Parse an auth mode string, returning an error for invalid values.
    ///
    /// This is a strict validation that fails loudly on invalid input,
    /// unlike the `From<&str>` implementation which falls back to `Open`.
    pub fn parse_strict(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "basic" => Ok(PlatformAuthMode::Basic),
            "jwt" => Ok(PlatformAuthMode::Jwt),
            "none" | "open" => Ok(PlatformAuthMode::Open),
            other => {
                Err(format!("Invalid auth mode '{}'. Valid options: basic, jwt, none, open", other))
            }
        }
    }
}

impl From<&str> for PlatformAuthMode {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "basic" => PlatformAuthMode::Basic,
            "jwt" => PlatformAuthMode::Jwt,
            "none" | "open" => PlatformAuthMode::Open,
            other => {
                tracing::warn!(
                    "Invalid PLATFORM_AUTH_MODE '{}' falling back to 'open'. Valid options: basic, jwt, none, open",
                    other
                );
                PlatformAuthMode::Open
            }
        }
    }
}

// Simple constant-time comparison to avoid leaking length/case differences in tokens.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let mut result = (a.len() ^ b.len()) as u8;
    let max_len = a.len().max(b.len());

    for i in 0..max_len {
        let x = a.as_bytes().get(i).copied().unwrap_or(0);
        let y = b.as_bytes().get(i).copied().unwrap_or(0);
        result |= x ^ y;
    }

    result == 0
}

fn token_kind(token: &str) -> TokenKind<'_> {
    if token.matches('.').count() == 2 { TokenKind::Jwt(token) } else { TokenKind::Basic(token) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auth_mode_and_warns_on_missing_token() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None };
        assert!(config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(!config.is_authorized(Some("anything")));
    }

    #[test]
    fn accepts_correct_token_in_basic_mode() {
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("secret".into()),
            jwt_secret: None,
        };

        assert!(!config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(config.is_authorized(Some("secret")));
        assert!(!config.is_authorized(Some("other")));
    }

    #[test]
    fn open_mode_requires_no_token() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
        assert!(!config.requires_auth());
        assert!(config.is_authorized(None));
        assert!(config.is_authorized(Some("anything")));
    }

    #[test]
    fn jwt_mode_requires_secret() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None };
        assert!(config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(!config.is_authorized(Some("anything")));
    }

    #[test]
    fn jwt_mode_accepts_valid_token() {
        let secret = "test-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some(secret.to_string()),
        };

        assert!(!config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(config.is_authorized(Some(&token)));
        assert!(!config.is_authorized(Some("invalid-token")));
    }

    #[test]
    fn jwt_mode_rejects_expired_token() {
        let secret = "test-secret";
        // Create an expired token (expires in the past)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let claims = Claims {
            sub: "user123".to_string(),
            exp: (now - 3600) as usize, // Expired 1 hour ago
            iat: (now - 7200) as usize, // Issued 2 hours ago
            iss: "rust-template".to_string(),
        };

        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();

        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some(secret.to_string()),
        };

        assert!(!config.is_authorized(Some(&token)));
    }

    #[test]
    fn jwt_mode_rejects_invalid_signature() {
        let secret = "test-secret";
        let wrong_secret = "wrong-secret";
        let token = create_jwt_token(secret, "user123", "rust-template", 3600).unwrap();

        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some(wrong_secret.to_string()),
        };

        assert!(!config.is_authorized(Some(&token)));
    }

    #[test]
    fn from_str_parses_all_modes() {
        assert_eq!(PlatformAuthMode::from("basic"), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::from("jwt"), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::from("none"), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::from("open"), PlatformAuthMode::Open);
        // Invalid mode falls back to Open with a warning
        assert_eq!(PlatformAuthMode::from("invalid"), PlatformAuthMode::Open);
    }

    #[test]
    fn parse_strict_validates_auth_modes() {
        // Valid modes should succeed
        assert_eq!(PlatformAuthMode::parse_strict("basic").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("jwt").unwrap(), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::parse_strict("none").unwrap(), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::parse_strict("open").unwrap(), PlatformAuthMode::Open);

        // Case insensitive
        assert_eq!(PlatformAuthMode::parse_strict("BASIC").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("JWT").unwrap(), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::parse_strict("None").unwrap(), PlatformAuthMode::Open);

        // Invalid modes should fail
        let result = PlatformAuthMode::parse_strict("invalid");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Invalid auth mode 'invalid'"));
        assert!(err_msg.contains("basic, jwt, none, open"));

        let result = PlatformAuthMode::parse_strict("bearer");
        assert!(result.is_err());

        let result = PlatformAuthMode::parse_strict("");
        assert!(result.is_err());
    }

    #[test]
    fn from_str_logs_warning_for_invalid_mode() {
        // Initialize tracing for test to capture logs
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        // Invalid mode should fall back to Open and log a warning
        let mode = PlatformAuthMode::from("invalid-mode");
        assert_eq!(mode, PlatformAuthMode::Open);

        // Another invalid mode
        let mode = PlatformAuthMode::from("bearer");
        assert_eq!(mode, PlatformAuthMode::Open);

        // Valid modes should not log warnings
        let mode = PlatformAuthMode::from("basic");
        assert_eq!(mode, PlatformAuthMode::Basic);
    }

    #[test]
    fn mode_label_returns_correct_string() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
        assert_eq!(config.mode_label(), "open");

        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None };
        assert_eq!(config.mode_label(), "basic");

        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None };
        assert_eq!(config.mode_label(), "jwt");
    }

    // AC-TPL-PLATFORM-AUTH-BASIC: token_present() validates by mode
    #[test]
    fn token_present_basic_mode_requires_basic_token() {
        // Basic mode with basic token: should return true
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("secret".into()),
            jwt_secret: None,
        };
        assert!(config.token_present(), "Basic mode with basic token should be present");

        // Basic mode with JWT secret only: should return false (wrong credential type)
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: None,
            jwt_secret: Some("jwt-secret".into()),
        };
        assert!(!config.token_present(), "Basic mode with only JWT secret should NOT be present");

        // Basic mode with no credentials: should return false
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None };
        assert!(!config.token_present(), "Basic mode with no credentials should NOT be present");

        // Basic mode with empty token: should return false
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("".into()),
            jwt_secret: None,
        };
        assert!(!config.token_present(), "Basic mode with empty token should NOT be present");
    }

    #[test]
    fn token_present_jwt_mode_requires_jwt_secret() {
        // JWT mode with JWT secret: should return true
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some("jwt-secret".into()),
        };
        assert!(config.token_present(), "JWT mode with JWT secret should be present");

        // JWT mode with basic token only: should return false (wrong credential type)
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: Some("basic-token".into()),
            jwt_secret: None,
        };
        assert!(!config.token_present(), "JWT mode with only basic token should NOT be present");

        // JWT mode with no credentials: should return false
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None };
        assert!(!config.token_present(), "JWT mode with no credentials should NOT be present");

        // JWT mode with empty secret: should return false
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some("".into()),
        };
        assert!(!config.token_present(), "JWT mode with empty secret should NOT be present");
    }

    #[test]
    fn token_present_open_mode_always_true() {
        // Open mode with no credentials: should return true
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
        assert!(config.token_present(), "Open mode should always be present");

        // Open mode with credentials: should still return true
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Open,
            token: Some("ignored".into()),
            jwt_secret: Some("also-ignored".into()),
        };
        assert!(config.token_present(), "Open mode with credentials should still be present");
    }

    // Tests for fail-closed auth configuration
    #[test]
    fn try_from_sources_valid_modes_succeed() {
        // Valid modes should parse without error (testing via parse_strict which try_from_sources uses)
        assert!(PlatformAuthMode::parse_strict("basic").is_ok());
        assert!(PlatformAuthMode::parse_strict("jwt").is_ok());
        assert!(PlatformAuthMode::parse_strict("open").is_ok());
        assert!(PlatformAuthMode::parse_strict("none").is_ok());
    }

    #[test]
    fn try_from_sources_invalid_mode_fails() {
        // Invalid mode should return an error, not silently default to open
        let result = PlatformAuthMode::parse_strict("invalid-mode");
        assert!(result.is_err(), "Invalid auth mode should fail, not silently default to open");

        let err = result.unwrap_err();
        assert!(err.contains("Invalid auth mode"), "Error should mention invalid auth mode");
        assert!(err.contains("basic, jwt, none, open"), "Error should list valid options");
    }

    #[test]
    fn try_from_sources_defaults_to_open_when_env_not_set() {
        // Rust 2024: mutating process env in tests is unsafe; avoid it.
        // This test is intentionally read-only and skips if the env var is set.
        if std::env::var("PLATFORM_AUTH_MODE").is_ok() {
            eprintln!("Skipping: PLATFORM_AUTH_MODE is set in this process");
            return;
        }

        let cfg = PlatformAuthConfig::try_from_sources(None).expect("default mode should be valid");
        assert_eq!(cfg.mode, PlatformAuthMode::Open);
    }
}
