use http_auth_config_inputs::{
    PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY, PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    PLATFORM_AUTH_MODE_CONFIG_KEY, PLATFORM_AUTH_MODE_ENV_VAR, PLATFORM_AUTH_TOKEN_CONFIG_KEY,
    PLATFORM_AUTH_TOKEN_ENV_VAR, PlatformAuthConfigInputs,
};
use http_auth_config_resolver::resolve_platform_auth_config;
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
fn scenario_integration_resolves_mode_and_secrets_from_inputs() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));
    let resolved = resolve_platform_auth_config(&inputs).expect("config should resolve");

    assert_eq!(resolved.mode.label(), "basic");
    assert_eq!(resolved.token.as_deref(), Some("cfg-token"));
    assert_eq!(resolved.jwt_secret.as_deref(), Some("cfg-secret"));
}

#[test]
fn scenario_integration_env_overrides_config_values() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "jwt");
    guard.set(PLATFORM_AUTH_TOKEN_ENV_VAR, "env-token");
    guard.set(PLATFORM_AUTH_JWT_SECRET_ENV_VAR, "env-secret");

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));
    let resolved = resolve_platform_auth_config(&inputs).expect("env should override");

    assert_eq!(resolved.mode.label(), "jwt");
    assert_eq!(resolved.token.as_deref(), Some("env-token"));
    assert_eq!(resolved.jwt_secret.as_deref(), Some("env-secret"));
}

#[test]
fn scenario_integration_invalid_mode_is_reported_from_end_to_end() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "definitely-not-valid");
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));
    let err = resolve_platform_auth_config(&inputs).expect_err("invalid mode should fail");

    assert!(err.contains("Invalid auth mode 'definitely-not-valid'"));
}
