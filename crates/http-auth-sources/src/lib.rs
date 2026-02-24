//! Source-resolution helpers for platform auth configuration.
//!
//! This crate owns only environmental + static config precedence rules for auth
//! material selection. Upstream policy crates decide how those resolved values are
//! used at authorization time.

#![forbid(unsafe_code)]

use http_auth_mode::PlatformAuthMode;

/// Resolved auth material prepared from the configured sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformAuthSourceConfig {
    /// Auth mode selected from source precedence.
    pub mode: PlatformAuthMode,
    /// Shared bearer-style token when Basic mode is configured.
    pub token: Option<String>,
    /// Shared JWT secret when JWT mode is configured.
    pub jwt_secret: Option<String>,
}

/// Build resolved auth material from environment/config values.
///
/// Precedence:
///
/// 1. Environment values win over config values.
/// 2. Config values win when env values are absent.
/// 3. `open` is used when no mode source exists.
pub fn resolve_auth_sources(
    mode_env: Option<&str>,
    token_env: Option<&str>,
    jwt_secret_env: Option<&str>,
    mode_config: Option<&str>,
    token_config: Option<&str>,
    jwt_secret_config: Option<&str>,
) -> Result<PlatformAuthSourceConfig, String> {
    let mode_raw = mode_env.or(mode_config).unwrap_or("open");
    let mode = PlatformAuthMode::parse_strict(mode_raw)?;

    let token = token_env.or(token_config).map(ToString::to_string);
    let jwt_secret = jwt_secret_env.or(jwt_secret_config).map(ToString::to_string);

    Ok(PlatformAuthSourceConfig { mode, token, jwt_secret })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn token_strategy() -> impl Strategy<Value = String> {
        "[A-Za-z0-9._-]{0,24}".prop_map(|token| token.to_string())
    }

    fn mode_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("open"),
            Just("none"),
            Just("basic"),
            Just("jwt"),
            Just("OPEN"),
            Just("NONE"),
            Just("BASIC"),
            Just("JWT"),
        ]
    }

    #[test]
    fn env_overrides_config_for_mode_and_credentials() {
        let result = resolve_auth_sources(
            Some("basic"),
            Some("env-token"),
            Some("env-secret"),
            Some("jwt"),
            Some("cfg-token"),
            Some("cfg-secret"),
        )
        .expect("env values should override");

        assert_eq!(result.mode, PlatformAuthMode::Basic);
        assert_eq!(result.token.as_deref(), Some("env-token"));
        assert_eq!(result.jwt_secret.as_deref(), Some("env-secret"));
    }

    #[test]
    fn config_fills_missing_env_values_and_defaults_to_open() {
        let result = resolve_auth_sources(
            None,
            None,
            None,
            Some("none"),
            Some("cfg-token"),
            Some("cfg-secret"),
        )
        .expect("config values should fill missing env values");

        assert_eq!(result.mode, PlatformAuthMode::Open);
        assert_eq!(result.token.as_deref(), Some("cfg-token"));
        assert_eq!(result.jwt_secret.as_deref(), Some("cfg-secret"));
    }

    #[test]
    fn config_missing_is_open_mode_when_absent() {
        let result = resolve_auth_sources(None, None, None, None, None, None)
            .expect("missing mode should default to open");

        assert_eq!(result.mode, PlatformAuthMode::Open);
        assert_eq!(result.token, None);
        assert_eq!(result.jwt_secret, None);
    }

    #[test]
    fn invalid_mode_fails_closed() {
        let err = resolve_auth_sources(Some("definitely-not-valid"), None, None, None, None, None)
            .expect_err("invalid mode must fail");

        assert!(err.contains("Invalid auth mode 'definitely-not-valid'"));
    }

    mod proptests {
        use super::*;

        proptest! {
            #[test]
            fn prop_env_mode_has_precedence(
                mode_env in prop::option::of(mode_strategy()),
                mode_cfg in prop::option::of(mode_strategy()),
                token_env in prop::option::of(token_strategy()),
                token_cfg in prop::option::of(token_strategy()),
                jwt_env in prop::option::of(token_strategy()),
                jwt_cfg in prop::option::of(token_strategy()),
            ) {
                let raw_mode = mode_env.unwrap_or(mode_cfg.unwrap_or("open"));

                let expected = PlatformAuthMode::parse_strict(raw_mode).unwrap();
                let actual = resolve_auth_sources(
                    mode_env,
                    token_env.as_deref(),
                    jwt_env.as_deref(),
                    mode_cfg,
                    token_cfg.as_deref(),
                    jwt_cfg.as_deref(),
                )
                .unwrap();

                prop_assert_eq!(actual.mode, expected);
                prop_assert_eq!(actual.token, token_env.or(token_cfg).map(|token| token.to_string()));
                prop_assert_eq!(actual.jwt_secret, jwt_env.or(jwt_cfg).map(|secret| secret.to_string()));
            }
        }
    }
}
