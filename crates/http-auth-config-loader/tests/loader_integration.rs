use http_auth_config_loader::load_platform_auth_config;
use http_auth_config_loader::{
    PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY, PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    PLATFORM_AUTH_MODE_CONFIG_KEY, PLATFORM_AUTH_MODE_ENV_VAR, PLATFORM_AUTH_TOKEN_CONFIG_KEY,
    PLATFORM_AUTH_TOKEN_ENV_VAR,
};
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
fn scenario_loader_uses_config_values_when_env_is_missing() {
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

    assert_eq!(auth.mode.label(), "basic");
    assert_eq!(auth.token.as_deref(), Some("cfg-token"));
    assert_eq!(auth.jwt_secret.as_deref(), Some("cfg-secret"));
}

#[test]
fn scenario_loader_environment_overrides_config_values() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "jwt");
    guard.set(PLATFORM_AUTH_TOKEN_ENV_VAR, "env-token");
    guard.set(PLATFORM_AUTH_JWT_SECRET_ENV_VAR, "env-secret");

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let auth = load_platform_auth_config(Some(&cfg)).expect("env should override");

    assert_eq!(auth.mode.label(), "jwt");
    assert_eq!(auth.token.as_deref(), Some("env-token"));
    assert_eq!(auth.jwt_secret.as_deref(), Some("env-secret"));
}

#[test]
fn scenario_loader_uses_open_as_safe_default_when_absent() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("none", "cfg-token", "cfg-secret");
    let auth = load_platform_auth_config(Some(&cfg)).expect("none should map to open");

    assert_eq!(auth.mode.label(), "open");
    assert_eq!(auth.token.as_deref(), Some("cfg-token"));
    assert_eq!(auth.jwt_secret.as_deref(), Some("cfg-secret"));
}

#[test]
fn scenario_loader_invalid_mode_is_reported_for_integration_chain() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.set(PLATFORM_AUTH_MODE_ENV_VAR, "definitely-not-valid");
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let err = load_platform_auth_config(Some(&cfg)).expect_err("invalid mode should fail");

    assert!(err.contains("Invalid auth mode 'definitely-not-valid'"));
}
