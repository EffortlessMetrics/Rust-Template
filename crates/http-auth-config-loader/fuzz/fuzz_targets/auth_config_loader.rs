#![no_main]

use arbitrary::Arbitrary;
use http_auth_config_inputs::{
    PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY, PLATFORM_AUTH_JWT_SECRET_ENV_VAR, PLATFORM_AUTH_MODE_CONFIG_KEY,
    PLATFORM_AUTH_MODE_ENV_VAR, PLATFORM_AUTH_TOKEN_CONFIG_KEY, PLATFORM_AUTH_TOKEN_ENV_VAR,
};
use http_auth_config_loader::load_platform_auth_config;
use libfuzzer_sys::fuzz_target;
use serde_yaml::Value;
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;
use testing::process::EnvVarGuard;

#[derive(Arbitrary, Debug)]
struct Input {
    mode_env: Option<String>,
    token_env: Option<String>,
    jwt_secret_env: Option<String>,
    mode_config: Option<String>,
    token_config: Option<String>,
    jwt_secret_config: Option<String>,
}

fn cfg_with(
    auth_mode: Option<&str>,
    auth_token: Option<&str>,
    jwt_secret: Option<&str>,
) -> ValidatedConfig {
    let mut settings = HashMap::new();
    if let Some(auth_mode) = auth_mode {
        settings.insert(PLATFORM_AUTH_MODE_CONFIG_KEY.to_string(), Value::String(auth_mode.to_string()));
    }

    let mut secrets = HashMap::new();
    if let Some(token) = auth_token {
        secrets.insert(PLATFORM_AUTH_TOKEN_CONFIG_KEY.to_string(), token.to_string());
    }
    if let Some(secret) = jwt_secret {
        secrets.insert(PLATFORM_AUTH_JWT_SECRET_CONFIG_KEY.to_string(), secret.to_string());
    }

    ValidatedConfig { env: Some("fuzz".to_string()), http_port: 8080, settings, secrets }
}

fuzz_target!(|input: Input| {
    let guard =
        EnvVarGuard::new(&[PLATFORM_AUTH_MODE_ENV_VAR, PLATFORM_AUTH_TOKEN_ENV_VAR, PLATFORM_AUTH_JWT_SECRET_ENV_VAR]);

    match input.mode_env {
        Some(mode) => guard.set(PLATFORM_AUTH_MODE_ENV_VAR, &mode),
        None => guard.remove(PLATFORM_AUTH_MODE_ENV_VAR),
    }
    match input.token_env {
        Some(token) => guard.set(PLATFORM_AUTH_TOKEN_ENV_VAR, &token),
        None => guard.remove(PLATFORM_AUTH_TOKEN_ENV_VAR),
    }
    match input.jwt_secret_env {
        Some(secret) => guard.set(PLATFORM_AUTH_JWT_SECRET_ENV_VAR, &secret),
        None => guard.remove(PLATFORM_AUTH_JWT_SECRET_ENV_VAR),
    }

    let cfg = cfg_with(input.mode_config.as_deref(), input.token_config.as_deref(), input.jwt_secret_config.as_deref());

    let _ = load_platform_auth_config(Some(&cfg));
});
