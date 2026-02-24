use http_auth_config::{PlatformAuthMode, try_from_sources};
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;
use testing::process::EnvVarGuard;

fn cfg_with(auth_mode: &str, auth_token: &str, jwt_secret: &str) -> ValidatedConfig {
    let mut settings = HashMap::new();
    settings
        .insert("platform.auth_mode".to_string(), serde_yaml::Value::String(auth_mode.to_string()));

    let mut secrets = HashMap::new();
    secrets.insert("platform.auth_token".to_string(), auth_token.to_string());
    secrets.insert("platform.jwt_secret".to_string(), jwt_secret.to_string());

    ValidatedConfig { env: Some("test".to_string()), http_port: 8080, settings, secrets }
}

#[test]
fn config_sources_used_when_env_unset() {
    let guard =
        EnvVarGuard::new(&["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN", "PLATFORM_JWT_SECRET"]);
    guard.remove("PLATFORM_AUTH_MODE");
    guard.remove("PLATFORM_AUTH_TOKEN");
    guard.remove("PLATFORM_JWT_SECRET");

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let auth = try_from_sources(Some(&cfg)).expect("config should parse");

    assert_eq!(auth.mode, PlatformAuthMode::Basic);
    assert_eq!(auth.token.as_deref(), Some("cfg-token"));
    assert_eq!(auth.jwt_secret.as_deref(), Some("cfg-secret"));
}

#[test]
fn env_overrides_config_values() {
    let guard =
        EnvVarGuard::new(&["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN", "PLATFORM_JWT_SECRET"]);
    guard.set("PLATFORM_AUTH_MODE", "jwt");
    guard.set("PLATFORM_AUTH_TOKEN", "env-token");
    guard.set("PLATFORM_JWT_SECRET", "env-secret");

    let cfg = cfg_with("basic", "cfg-token", "cfg-secret");
    let auth = try_from_sources(Some(&cfg)).expect("config should parse");

    assert_eq!(auth.mode, PlatformAuthMode::Jwt);
    assert_eq!(auth.token.as_deref(), Some("env-token"));
    assert_eq!(auth.jwt_secret.as_deref(), Some("env-secret"));
}

#[test]
fn invalid_mode_fails_closed() {
    let guard =
        EnvVarGuard::new(&["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN", "PLATFORM_JWT_SECRET"]);
    guard.set("PLATFORM_AUTH_MODE", "definitely-not-valid");
    guard.remove("PLATFORM_AUTH_TOKEN");
    guard.remove("PLATFORM_JWT_SECRET");

    let error = try_from_sources(None).expect_err("invalid mode must error");
    assert!(
        error.contains("Invalid auth mode 'definitely-not-valid'"),
        "unexpected error: {error}"
    );
}

#[test]
fn none_alias_maps_to_open_mode() {
    let guard =
        EnvVarGuard::new(&["PLATFORM_AUTH_MODE", "PLATFORM_AUTH_TOKEN", "PLATFORM_JWT_SECRET"]);
    guard.remove("PLATFORM_AUTH_MODE");
    guard.remove("PLATFORM_AUTH_TOKEN");
    guard.remove("PLATFORM_JWT_SECRET");

    let cfg = cfg_with("none", "cfg-token", "cfg-secret");
    let auth = try_from_sources(Some(&cfg)).expect("config should parse");

    assert_eq!(auth.mode, PlatformAuthMode::Open);
    assert!(!auth.requires_auth());
}
