use http_auth_config_inputs::{
    PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY, PLATFORM_AUTH_MODE_CONFIG_KEY, PLATFORM_AUTH_MODE_ENV_VAR,
    PLATFORM_AUTH_TOKEN_CONFIG_KEY, PLATFORM_AUTH_TOKEN_ENV_VAR,
};
use http_auth_config_inputs::{PLATFORM_AUTH_JWT_SECRET_ENV_VAR, PlatformAuthConfigInputs};
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
fn scenario_env_wins_for_mode_and_credentials() {
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

    assert_eq!(inputs.mode(), Some("basic"));
    assert_eq!(inputs.token(), Some("env-token"));
    assert_eq!(inputs.jwt_secret(), Some("env-secret"));
    assert_eq!(inputs.mode_config.as_deref(), Some("jwt"));
    assert_eq!(inputs.token_config.as_deref(), Some("cfg-token"));
    assert_eq!(inputs.jwt_secret_config.as_deref(), Some("cfg-secret"));
}

#[test]
fn scenario_config_backfills_when_environment_is_missing() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("jwt", "cfg-token", "cfg-secret");
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));

    assert_eq!(inputs.mode(), Some("jwt"));
    assert_eq!(inputs.token(), Some("cfg-token"));
    assert_eq!(inputs.jwt_secret(), Some("cfg-secret"));
}

#[test]
fn scenario_invalid_config_value_is_preserved_for_resolvers_to_validate() {
    let guard = EnvVarGuard::new(&[
        PLATFORM_AUTH_MODE_ENV_VAR,
        PLATFORM_AUTH_TOKEN_ENV_VAR,
        PLATFORM_AUTH_JWT_SECRET_ENV_VAR,
    ]);
    guard.remove(PLATFORM_AUTH_MODE_ENV_VAR);
    guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR);
    guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR);

    let cfg = cfg_with("definitely-not-valid", "", "");
    let inputs = PlatformAuthConfigInputs::collect_from_runtime_config(Some(&cfg));

    assert_eq!(inputs.mode(), Some("definitely-not-valid"));
}
