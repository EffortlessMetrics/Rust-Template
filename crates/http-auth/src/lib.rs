//! Platform HTTP authentication primitives.
//!
//! This crate owns credential sourcing and auth-mode policy decisions.
//! `PlatformAuthMode` parsing lives in `http-auth-mode` and is re-exported here
//! for compatibility.

pub use http_auth_mode::PlatformAuthMode;
pub use http_auth_verifier::Claims;
use http_auth_verifier::authorize_token;
#[cfg(test)]
use jsonwebtoken::{EncodingKey, Header, encode};
use spec_runtime::ValidatedConfig;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

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

impl PlatformAuthConfig {
    /// Build auth config from env vars and optional validated config.
    ///
    /// Precedence is environment-first, then config.
    ///
    /// Fails closed when auth mode is invalid.
    pub fn try_from_sources(config: Option<&ValidatedConfig>) -> Result<Self, String> {
        let mode_raw = std::env::var("PLATFORM_AUTH_MODE")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("platform.auth_mode"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| "open".to_string());

        let mode = PlatformAuthMode::parse_strict(&mode_raw)?;

        let token = std::env::var("PLATFORM_AUTH_TOKEN")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.auth_token").cloned()));

        let jwt_secret = std::env::var("PLATFORM_JWT_SECRET")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.jwt_secret").cloned()));

        Ok(Self { mode, token, jwt_secret })
    }

    /// True when mode requires auth checks (`basic` or `jwt`).
    pub fn requires_auth(&self) -> bool {
        matches!(self.mode, PlatformAuthMode::Basic | PlatformAuthMode::Jwt)
    }

    /// True when mode requires auth and matching credential is configured.
    pub fn can_enforce_auth(&self) -> bool {
        match self.mode {
            PlatformAuthMode::Open => false,
            PlatformAuthMode::Basic => self.token.as_ref().is_some_and(|t| !t.is_empty()),
            PlatformAuthMode::Jwt => self.jwt_secret.as_ref().is_some_and(|s| !s.is_empty()),
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
        self.token.as_ref().is_some_and(|t| !t.is_empty())
    }

    fn has_jwt_secret(&self) -> bool {
        self.jwt_secret.as_ref().is_some_and(|s| !s.is_empty())
    }
}

#[cfg(test)]
fn create_jwt_token(
    secret: &str,
    subject: &str,
    issuer: &str,
    expires_in_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let claims = Claims {
        sub: subject.to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        iss: issuer.to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_mode_authorization_behaves_as_expected() {
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("secret".into()),
            jwt_secret: None,
        };

        assert!(config.requires_auth());
        assert!(config.can_enforce_auth());
        assert!(config.is_authorized(Some("secret")));
        assert!(!config.is_authorized(Some("other")));
        assert!(!config.is_authorized(None));
    }

    #[test]
    fn basic_mode_accepts_literal_token_with_jwt_like_shape() {
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("dot.token.value".into()),
            jwt_secret: Some("unused-secret".into()),
        };

        assert!(config.is_authorized(Some("dot.token.value")));
    }

    #[test]
    fn open_mode_is_always_authorized() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };

        assert!(!config.requires_auth());
        assert!(!config.can_enforce_auth());
        assert!(config.is_authorized(None));
        assert!(config.is_authorized(Some("anything")));
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

        assert!(config.requires_auth());
        assert!(config.can_enforce_auth());
        assert!(config.is_authorized(Some(&token)));
        assert!(!config.is_authorized(Some("invalid-token")));
    }

    #[test]
    fn jwt_mode_rejects_expired_token() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let claims = Claims {
            sub: "user123".to_string(),
            exp: now - 3600,
            iat: now - 7200,
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
        let token = create_jwt_token("correct-secret", "user123", "rust-template", 3600).unwrap();
        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some("wrong-secret".to_string()),
        };

        assert!(!config.is_authorized(Some(&token)));
    }

    #[test]
    fn jwt_mode_rejects_missing_claims() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let encoding_key = EncodingKey::from_secret(secret.as_ref());

        // Empty issuer
        let token_missing_issuer = encode(
            &Header::default(),
            &serde_json::json!({
                "sub": "user123",
                "exp": now + 3600,
                "iat": now,
                "iss": ""
            }),
            &encoding_key,
        )
        .unwrap();

        // Empty subject
        let token_missing_subject = encode(
            &Header::default(),
            &serde_json::json!({
                "sub": "",
                "exp": now + 3600,
                "iat": now,
                "iss": "rust-template"
            }),
            &encoding_key,
        )
        .unwrap();

        let config = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some(secret.to_string()),
        };

        assert!(!config.is_authorized(Some(&token_missing_issuer)));
        assert!(!config.is_authorized(Some(&token_missing_subject)));
    }

    #[test]
    fn jwt_mode_rejects_iat_too_far_in_future() {
        let secret = "test-secret";
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let claims = Claims {
            sub: "user123".to_string(),
            exp: now + 3600,
            iat: now + 301,
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
    fn parse_strict_validates_auth_modes() {
        assert_eq!(PlatformAuthMode::parse_strict("basic").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("jwt").unwrap(), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::parse_strict("none").unwrap(), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::parse_strict("open").unwrap(), PlatformAuthMode::Open);

        assert_eq!(PlatformAuthMode::parse_strict("BASIC").unwrap(), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::parse_strict("JWT").unwrap(), PlatformAuthMode::Jwt);

        assert!(PlatformAuthMode::parse_strict("invalid").is_err());
        assert!(PlatformAuthMode::parse_strict("bearer").is_err());
        assert!(PlatformAuthMode::parse_strict("").is_err());
    }

    #[test]
    fn from_str_falls_back_to_open_for_invalid() {
        assert_eq!(PlatformAuthMode::from("basic"), PlatformAuthMode::Basic);
        assert_eq!(PlatformAuthMode::from("jwt"), PlatformAuthMode::Jwt);
        assert_eq!(PlatformAuthMode::from("open"), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::from("none"), PlatformAuthMode::Open);
        assert_eq!(PlatformAuthMode::from("invalid"), PlatformAuthMode::Open);
    }

    #[test]
    fn mode_label_returns_expected_values() {
        assert_eq!(
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None }
                .mode_label(),
            "open"
        );
        assert_eq!(
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None }
                .mode_label(),
            "basic"
        );
        assert_eq!(
            PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None }
                .mode_label(),
            "jwt"
        );
    }

    #[test]
    fn token_present_respects_mode() {
        let basic_with_token = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("secret".into()),
            jwt_secret: None,
        };
        assert!(basic_with_token.token_present());

        let basic_with_only_secret = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: None,
            jwt_secret: Some("jwt-secret".into()),
        };
        assert!(!basic_with_only_secret.token_present());

        let jwt_with_secret = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: None,
            jwt_secret: Some("jwt-secret".into()),
        };
        assert!(jwt_with_secret.token_present());

        let jwt_with_only_token = PlatformAuthConfig {
            mode: PlatformAuthMode::Jwt,
            token: Some("basic-token".into()),
            jwt_secret: None,
        };
        assert!(!jwt_with_only_token.token_present());

        let open =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
        assert!(open.token_present());
    }

    #[test]
    fn warn_if_misconfigured_only_when_required() {
        let missing_basic =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None, jwt_secret: None };
        assert!(missing_basic.warn_if_misconfigured());

        let missing_jwt =
            PlatformAuthConfig { mode: PlatformAuthMode::Jwt, token: None, jwt_secret: None };
        assert!(missing_jwt.warn_if_misconfigured());

        let open =
            PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };
        assert!(!open.warn_if_misconfigured());
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_basic_mode_requires_exact_token(
                token in "[A-Za-z0-9_-]{1,32}",
                candidate in "[A-Za-z0-9_-]{1,32}",
            ) {
                let cfg = PlatformAuthConfig {
                    mode: PlatformAuthMode::Basic,
                    token: Some(token.clone()),
                    jwt_secret: None,
                };

                prop_assert!(cfg.is_authorized(Some(&token)));
                prop_assert_eq!(cfg.is_authorized(Some(&candidate)), token == candidate);
            }

            #[test]
            fn prop_mode_parser_case_insensitive(
                raw in prop_oneof![
                    Just("basic".to_string()),
                    Just("jwt".to_string()),
                    Just("open".to_string()),
                    Just("none".to_string())
                ],
                upper in any::<bool>(),
            ) {
                let input = if upper { raw.to_ascii_uppercase() } else { raw.clone() };
                let parsed = PlatformAuthMode::parse_strict(&input).unwrap();

                let expected = match raw.as_str() {
                    "basic" => PlatformAuthMode::Basic,
                    "jwt" => PlatformAuthMode::Jwt,
                    "open" | "none" => PlatformAuthMode::Open,
                    _ => unreachable!(),
                };

                prop_assert_eq!(parsed, expected);
            }

            #[test]
            fn prop_open_mode_always_authorized(candidate in ".*") {
                let cfg = PlatformAuthConfig {
                    mode: PlatformAuthMode::Open,
                    token: None,
                    jwt_secret: None,
                };

                prop_assert!(cfg.is_authorized(None));
                prop_assert!(cfg.is_authorized(Some(&candidate)));
            }
        }
    }
}
