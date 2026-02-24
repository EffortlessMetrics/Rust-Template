//! Resolve platform authentication inputs into runtime policy config.

#![forbid(unsafe_code)]

use http_auth_config_inputs::PlatformAuthConfigInputs;
use http_auth_policy::PlatformAuthConfig;
use http_auth_sources::resolve_auth_sources;

/// Resolve collected auth inputs into a concrete auth policy config.
///
/// Resolution precedence and mode parsing stay inside `http-auth-sources`.
/// This crate only assembles the final policy object.
pub fn resolve_platform_auth_config(
    inputs: &PlatformAuthConfigInputs,
) -> Result<PlatformAuthConfig, String> {
    let source = resolve_auth_sources(
        inputs.mode_env.as_deref(),
        inputs.token_env.as_deref(),
        inputs.jwt_secret_env.as_deref(),
        inputs.mode_config.as_deref(),
        inputs.token_config.as_deref(),
        inputs.jwt_secret_config.as_deref(),
    )?;

    Ok(PlatformAuthConfig::new(source.mode, source.token, source.jwt_secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn build_inputs(
        mode_env: Option<&str>,
        token_env: Option<&str>,
        jwt_secret_env: Option<&str>,
        mode_config: Option<&str>,
        token_config: Option<&str>,
        jwt_secret_config: Option<&str>,
    ) -> PlatformAuthConfigInputs {
        PlatformAuthConfigInputs {
            mode_env: mode_env.map(ToString::to_string),
            token_env: token_env.map(ToString::to_string),
            jwt_secret_env: jwt_secret_env.map(ToString::to_string),
            mode_config: mode_config.map(ToString::to_string),
            token_config: token_config.map(ToString::to_string),
            jwt_secret_config: jwt_secret_config.map(ToString::to_string),
        }
    }

    #[test]
    fn scenario_resolve_prefers_env_sources_for_mode_and_credentials() {
        let inputs = build_inputs(
            Some("basic"),
            Some("env-token"),
            Some("env-secret"),
            Some("jwt"),
            Some("cfg-token"),
            Some("cfg-secret"),
        );
        let resolved = resolve_platform_auth_config(&inputs).expect("valid mode should resolve");

        assert_eq!(resolved.mode.label(), "basic");
        assert_eq!(resolved.token.as_deref(), Some("env-token"));
        assert_eq!(resolved.jwt_secret.as_deref(), Some("env-secret"));
    }

    #[test]
    fn scenario_resolve_uses_config_when_env_is_missing() {
        let inputs =
            build_inputs(None, None, None, Some("jwt"), Some("cfg-token"), Some("cfg-secret"));
        let resolved = resolve_platform_auth_config(&inputs).expect("config should backfill");

        assert_eq!(resolved.mode.label(), "jwt");
        assert_eq!(resolved.token.as_deref(), Some("cfg-token"));
        assert_eq!(resolved.jwt_secret.as_deref(), Some("cfg-secret"));
    }

    #[test]
    fn scenario_none_alias_is_treated_as_open() {
        let inputs = build_inputs(Some("none"), None, None, None, None, None);
        let resolved = resolve_platform_auth_config(&inputs).expect("none should map to open");

        assert_eq!(resolved.mode.label(), "open");
        assert!(!resolved.requires_auth());
    }

    #[test]
    fn scenario_invalid_mode_is_rejected() {
        let inputs = build_inputs(Some("definitely-not-valid"), None, None, None, None, None);
        let err = resolve_platform_auth_config(&inputs).expect_err("invalid mode must fail");
        assert!(err.contains("Invalid auth mode 'definitely-not-valid'"));
    }

    #[cfg(test)]
    mod proptests {
        use super::*;

        fn token_strategy() -> impl Strategy<Value = String> {
            "[A-Za-z0-9._~-]{0,24}".prop_map(|token| token.to_string())
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

        proptest! {
            #[test]
            fn prop_env_or_config_wins_with_credentials(
                mode_env in prop::option::of(mode_strategy()),
                mode_config in prop::option::of(mode_strategy()),
                token_env in prop::option::of(token_strategy()),
                token_config in prop::option::of(token_strategy()),
                jwt_env in prop::option::of(token_strategy()),
                jwt_config in prop::option::of(token_strategy()),
            ) {
                let expected_mode = mode_env.clone().or(mode_config.clone()).unwrap_or("open".to_string());
                let expected_mode = match expected_mode.to_ascii_lowercase().as_str() {
                    "basic" => "basic",
                    "jwt" => "jwt",
                    "none" | "open" => "open",
                    _ => unreachable!(),
                };
                let expected_token = token_env.clone().or_else(|| token_config.clone());
                let expected_jwt = jwt_env.clone().or_else(|| jwt_config.clone());

                let inputs = PlatformAuthConfigInputs {
                    mode_env,
                    token_env,
                    jwt_secret_env: jwt_env,
                    mode_config,
                    token_config,
                    jwt_secret_config: jwt_config,
                };

                let resolved = resolve_platform_auth_config(&inputs).expect("mode strategy is valid");
                assert_eq!(resolved.mode.label(), expected_mode);
                assert_eq!(resolved.token, expected_token);
                assert_eq!(resolved.jwt_secret, expected_jwt);
            }

            #[test]
            fn prop_open_mode_defaults_when_no_mode_sources(
                token_env in prop::option::of(token_strategy()),
                token_config in prop::option::of(token_strategy()),
                jwt_env in prop::option::of(token_strategy()),
                jwt_config in prop::option::of(token_strategy()),
            ) {
                let inputs = build_inputs(
                    None,
                    token_env.as_deref(),
                    jwt_env.as_deref(),
                    None,
                    token_config.as_deref(),
                    jwt_config.as_deref(),
                );

                let resolved = resolve_platform_auth_config(&inputs).expect("open is always valid");
                let expected_token = token_env.clone().or(token_config.clone());
                let expected_jwt = jwt_env.clone().or(jwt_config.clone());

                assert_eq!(resolved.mode.label(), "open");
                assert_eq!(resolved.token, expected_token);
                assert_eq!(resolved.jwt_secret, expected_jwt);
            }
        }
    }
}
