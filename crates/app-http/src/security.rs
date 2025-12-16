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

        let jwt_secret = std::env::var("PLATFORM_JWT_SECRET")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.jwt_secret").cloned()));

        Self { mode: PlatformAuthMode::from(mode_raw.as_str()), token, jwt_secret }
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
            TokenKind::Basic(candidate) => self
                .token
                .as_deref()
                .map_or(false, |expected| constant_time_eq(expected, candidate)),
            TokenKind::Jwt(candidate) => self
                .jwt_secret
                .as_deref()
                .map_or(false, |secret| validate_jwt_token(candidate, secret)),
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            PlatformAuthMode::Open => "open",
            PlatformAuthMode::Basic => "basic",
            PlatformAuthMode::Jwt => "jwt",
        }
    }

    pub fn token_present(&self) -> bool {
        let has_basic = self.has_basic_token();
        let has_jwt = self.has_jwt_secret();

        match self.mode {
            PlatformAuthMode::Basic | PlatformAuthMode::Jwt => has_basic || has_jwt,
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
    validation.validate_exp = true;
    validation.leeway = 0;

    let decoding_key = DecodingKey::from_secret(secret.as_ref());

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(_) => true,
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            false
        }
    }
}

/// Create a JWT token with the provided secret and claims
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

impl From<&str> for PlatformAuthMode {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "basic" => PlatformAuthMode::Basic,
            "jwt" => PlatformAuthMode::Jwt,
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

fn token_kind(token: &str) -> TokenKind<'_> {
    if token.split('.').count() == 3 { TokenKind::Jwt(token) } else { TokenKind::Basic(token) }
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
        assert_eq!(PlatformAuthMode::from("invalid"), PlatformAuthMode::Open);
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
}
