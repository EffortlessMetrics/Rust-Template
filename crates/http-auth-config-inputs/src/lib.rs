//! Environment and validated-config extraction for platform authentication values.
//!
//! This crate intentionally owns only input-precedence concerns. It reads from
//! environment variables and runtime config and yields raw source material for
//! downstream auth-source resolution.

#![forbid(unsafe_code)]

use spec_runtime::ValidatedConfig;

/// Environment variable used for auth mode selection.
pub const PLATFORM_AUTH_MODE_ENV_VAR: &str = "PLATFORM_AUTH_MODE";
/// Environment variable used for static token auth.
pub const PLATFORM_AUTH_TOKEN_ENV_VAR: &str = "PLATFORM_AUTH_TOKEN";
/// Environment variable used for JWT auth.
pub const PLATFORM_AUTH_JWT_SECRET_ENV_VAR: &str = "PLATFORM_JWT_SECRET";

/// Runtime config setting key for auth mode.
pub const PLATFORM_AUTH_MODE_CONFIG_KEY: &str = "platform.auth_mode";
/// Runtime config setting key for static token auth.
pub const PLATFORM_AUTH_TOKEN_CONFIG_KEY: &str = "platform.auth_token";
/// Runtime config setting key for JWT secret.
pub const PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY: &str = "platform.jwt_secret";

/// Raw auth inputs from environment and validated runtime config.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformAuthConfigInputs {
    /// Mode from env (highest precedence) or config.
    pub mode_env: Option<String>,
    /// Token from env (highest precedence) or config.
    pub token_env: Option<String>,
    /// JWT secret from env (highest precedence) or config.
    pub jwt_secret_env: Option<String>,
    /// Mode from config.
    pub mode_config: Option<String>,
    /// Token from config.
    pub token_config: Option<String>,
    /// JWT secret from config.
    pub jwt_secret_config: Option<String>,
}

impl PlatformAuthConfigInputs {
    /// Collect platform auth input sources from environment and optional runtime config.
    pub fn collect_from_runtime_config(config: Option<&ValidatedConfig>) -> Self {
        let mode_env = std::env::var(PLATFORM_AUTH_MODE_ENV_VAR).ok();
        let token_env = std::env::var(PLATFORM_AUTH_TOKEN_ENV_VAR).ok();
        let jwt_secret_env = std::env::var(PLATFORM_AUTH_JWT_SECRET_ENV_VAR).ok();

        let mode_config = config
            .and_then(|cfg| cfg.settings.get(PLATFORM_AUTH_MODE_CONFIG_KEY))
            .and_then(|value| value.as_str())
            .map(ToString::to_string);

        let token_config =
            config.and_then(|cfg| cfg.secrets.get(PLATFORM_AUTH_TOKEN_CONFIG_KEY)).cloned();

        let jwt_secret_config =
            config.and_then(|cfg| cfg.secrets.get(PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY)).cloned();

        Self { mode_env, token_env, jwt_secret_env, mode_config, token_config, jwt_secret_config }
    }

    /// Effective mode input with env-first precedence.
    pub fn mode(&self) -> Option<&str> {
        self.mode_env.as_deref().or(self.mode_config.as_deref())
    }

    /// Effective token input with env-first precedence.
    pub fn token(&self) -> Option<&str> {
        self.token_env.as_deref().or(self.token_config.as_deref())
    }

    /// Effective JWT secret input with env-first precedence.
    pub fn jwt_secret(&self) -> Option<&str> {
        self.jwt_secret_env.as_deref().or(self.jwt_secret_config.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use spec_runtime::ValidatedConfig;
    use std::collections::HashMap;
    use testing::process::EnvVarGuard;

    fn cfg_with(auth_mode: &str, auth_token: &str, jwt_secret: &str) -> ValidatedConfig {
        let mut settings = HashMap::new();
        settings.insert(
            PLATFORM_AUTH_MODE_CONFIG_KEY.to_string(),
            serde_yaml::Value::String(auth_mode.to_string()),
        );

        let mut secrets = HashMap::new();
        secrets.insert(PLATFORM_AUTH_TOKEN_CONFIG_KEY.to_string(), auth_token.to_string());
        secrets.insert(PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY.to_string(), jwt_secret.to_string());

        ValidatedConfig { env: Some("test".to_string()), http_port: 8080, settings, secrets }
    }

    #[test]
    fn collect_env_and_config_sources_with_precedence_for_mode_and_credentials() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "basic");
        guard.set(PLATFORM_AUTH_TOKEN_ENV_VAR, "env-token");
        guard.set(PLATFORM_AUTH_JWT_SECRET_ENV_VAR, "env-secret");

        let cfg = cfg_with("jwt", "cfg-token", "cfg-secret");
        let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));

        assert_eq!(inputs.mode_env.as_deref(), Some("basic"));
        assert_eq!(inputs.mode_config.as_deref(), Some("jwt"));
        assert_eq!(inputs.token(), Some("env-token"));
        assert_eq!(inputs.token_config.as_deref(), Some("cfg-token"));
        assert_eq!(inputs.jwt_secret(), Some("env-secret"));
        assert_eq!(inputs.jwt_secret_config.as_deref(), Some("cfg-secret"));
    }

    #[test]
    fn collect_collects_config_when_env_missing() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
        guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
        guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

        let cfg = cfg_with("none", "cfg-token", "cfg-secret");
        let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));

        assert_eq!(inputs.mode(), Some("none"));
        assert_eq!(inputs.token(), Some("cfg-token"));
        assert_eq!(inputs.jwt_secret(), Some("cfg-secret"));
    }

    #[test]
    fn collect_keeps_none_when_env_and_config_absent() {
        let guard = EnvVarGuard::new(&[
            PLATFORM_AUTH_MODE_ENV_VAR,
            PLATFORM_AUTH_TOKEN_ENV_VAR,
            PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
        ]);
        guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
        guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
        guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

        let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(None);

        assert_eq!(inputs.mode(), None);
        assert_eq!(inputs.token(), None);
        assert_eq!(inputs.jwt_secret(), None);
    }

    fn token_strategy() -> impl Strategy<Value = String> {
        "[A-Za-z0-9._~-]{0,24}".prop_map(|token| token.to_string())
    }

    fn precedence_mode_strategy() -> impl Strategy<Value = String> {
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

    #[cfg(test)]
    mod proptests {
        use super::*;

        proptest! {
            #[test]
            fn prop_env_mode_takes_precedence(
                mode_env in prop::option::of(precedence_mode_strategy()),
                mode_config in prop::option::of(precedence_mode_strategy()),
            ) {
                let inputs = PlatformAuthConfigInputs {
                    mode_env,
                    token_env: None,
                    jwt_secret_env: None,
                    mode_config,
                    token_config: None,
                    jwt_secret_config: None,
                };

                let expected = inputs.mode_env.as_deref().or(inputs.mode_config.as_deref());
                prop_assert_eq!(inputs.mode(), expected);
            }

            #[test]
            fn prop_env_credentials_precedence(
                token_env in prop::option::of(token_strategy()),
                token_config in prop::option::of(token_strategy()),
                jwt_env in prop::option::of(token_strategy()),
                jwt_config in prop::option::of(token_strategy()),
            ) {
                let inputs = PlatformAuthConfigInputs {
                    mode_env: None,
                    mode_config: None,
                    token_env,
                    token_config,
                    jwt_secret_env: jwt_env,
                    jwt_secret_config: jwt_config,
                };

                let expected_token = inputs.token_env.as_deref().or(inputs.token_config.as_deref());
                let expected_jwt = inputs.jwt_secret_env.as_deref().or(inputs.jwt_secret_config.as_deref());

                prop_assert_eq!(inputs.token(), expected_token);
                prop_assert_eq!(inputs.jwt_secret(), expected_jwt);
            }
        }
    }
}
