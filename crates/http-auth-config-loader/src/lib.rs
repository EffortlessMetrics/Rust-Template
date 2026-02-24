//! SRP boundary for loading `PlatformAuthConfig` from environment and runtime config.
//!
//! This crate contains a single responsibility: load auth configuration using the
//! input collection and resolver crates and return a concrete `PlatformAuthConfig`.

#![forbid(unsafe_code)]

pub use http_auth_config_inputs::{
    PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY, PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    PLATFORM_AUTH_MODE_CONFIG_KEY, PLATFORM_AUTH_MODE_ENV_VAR, PLATFORM_AUTH_TOKEN_CONFIG_KEY,
    PLATFORM_AUTH_TOKEN_ENV_VAR, PlatformAuthConfigInputs,
};
pub use http_auth_config_resolver::resolve_platform_auth_config;
pub use http_auth_policy::{PlatformAuthConfig, PlatformAuthMode};
pub use http_auth_verifier::Claims;

use spec_runtime::ValidatedConfig;

/// Build platform auth configuration from environment variables and validated config.
///
/// Environment values have precedence over runtime config values.
pub fn load_platform_auth_config(
    config: Option<&ValidatedConfig>,
) -> Result<PlatformAuthConfig, String> {
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(config);
    resolve_platform_auth_config(&inputs)
}

/// Backward-compatible alias for loading config from env + runtime config sources.
pub fn try_from_sources(config: Option<&ValidatedConfig>) -> Result<PlatformAuthConfig, String> {
    load_platform_auth_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_yaml::Value;
    use spec_runtime::ValidatedConfig;
    use std::collections::HashMap;
    use testing::process::EnvVarGuard;

    fn cfg_with(auth_mode: &str, auth_token: &str, jwt_secret: &str) -> ValidatedConfig {
        let mut settings = HashMap::new();
        settings.insert(
            PLATFORM_AUTH_MODE_CONFIG_KEY.to_string(),
            Value::String(auth_mode.to_string()),
        );

        let mut secrets = HashMap::new();
        secrets.insert(PLATFORM_AUTH_TOKEN_CONFIG_KEY.to_string(), auth_token.to_string());
        secrets.insert(PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY.to_string(), jwt_secret.to_string());

        ValidatedConfig { env: Some("test".to_string()), http_port: 8080, settings, secrets }
    }

    #[test]
    fn config_values_backfill_when_env_is_missing() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
        guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
        guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

        let cfg = cfg_with("basic", "cfg-token", "cfg-secret");

        let auth = load_platform_auth_config(Some(&cfg)).expect("config should parse");

        assert_eq!(auth.mode, PlatformAuthMode::Basic);
        assert_eq!(auth.token.as_deref(), Some("cfg-token"));
        assert_eq!(auth.jwt_secret.as_deref(), Some("cfg-secret"));
    }

    #[test]
    fn env_overrides_config_values() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "jwt");
        guard.set(PLATFORM_AUTH_TOKEN_ENV_VAR, "env-token");
        guard.set(PLATFORM_AUTH_JWT_SECRET_ENV_VAR, "env-secret");

        let cfg = cfg_with("basic", "cfg-token", "cfg-secret");

        let auth = load_platform_auth_config(Some(&cfg)).expect("env values should parse");

        assert_eq!(auth.mode, PlatformAuthMode::Jwt);
        assert_eq!(auth.token.as_deref(), Some("env-token"));
        assert_eq!(auth.jwt_secret.as_deref(), Some("env-secret"));
    }

    #[test]
    fn invalid_mode_fails_closed() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "definitely-not-valid");
        guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
        guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

        let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
        let err = load_platform_auth_config(Some(&cfg)).expect_err("invalid env mode must fail");

        assert!(err.contains("Invalid auth mode 'definitely-not-valid'"));
    }

    fn mode_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("open".to_string()),
            Just("none".to_string()),
            Just("basic".to_string()),
            Just("jwt".to_string()),
            Just("OPEN".to_string()),
            Just("NONE".to_string()),
            Just("BASIC".to_string()),
            Just("JWT".to_string()),
        ]
    }

    mod proptests {
        use super::*;

        fn token_strategy() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~-]{0,24}".prop_map(|token| token.to_string())
        }

        proptest! {
            #[test]
            fn prop_config_mode_is_lowercased_and_mapped(
                mode in mode_strategy(),
            ) {
                let cfg = super::cfg_with(mode.as_str(), "", "");
                let guard = EnvVarGuard::new(&[
                    PLATFORM_AUTH_MODE_ENV_VAR,
                    PLATFORM_AUTH_TOKEN_ENV_VAR,
                    PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
                ]);
                guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
                guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
                guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

                let resolved = load_platform_auth_config(Some(&cfg)).expect("mode strategy should resolve");
                let expected = match mode.to_ascii_lowercase().as_str() {
                    "none" | "open" => PlatformAuthMode::Open,
                    "basic" => PlatformAuthMode::Basic,
                    "jwt" => PlatformAuthMode::Jwt,
                    _ => unreachable!(),
                };

                prop_assert_eq!(resolved.mode, expected);
            }

            #[test]
            fn prop_config_credentials_are_preserved(
                token in token_strategy(),
                jwt_secret in token_strategy(),
            ) {
                let cfg = super::cfg_with("open", token.as_str(), jwt_secret.as_str());
                let guard = EnvVarGuard::new(&[
                    PLATFORM_AUTH_MODE_ENV_VAR,
                    PLATFORM_AUTH_TOKEN_ENV_VAR,
                    PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
                ]);
                guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
                guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
                guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

                let resolved = load_platform_auth_config(Some(&cfg)).expect("credentials should pass through");

                prop_assert_eq!(resolved.token, Some(token));
                prop_assert_eq!(resolved.jwt_secret, Some(jwt_secret));
            }
        }
    }
}
