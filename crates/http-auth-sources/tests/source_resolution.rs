use http_auth_sources::resolve_auth_sources;

#[test]
fn scenario_env_and_config_can_be_resolved_for_authorization_flow() {
    let resolved = resolve_auth_sources(
        Some("basic"),
        Some("token-from-env"),
        Some("env-jwt"),
        Some("jwt"),
        Some("token-from-config"),
        Some("config-jwt"),
    )
    .expect("expected valid mode to resolve");

    assert_eq!(resolved.mode.label(), "basic");
    assert_eq!(resolved.token.as_deref(), Some("token-from-env"));
    assert_eq!(resolved.jwt_secret.as_deref(), Some("env-jwt"));
}

#[test]
fn scenario_config_backfills_when_environment_is_missing() {
    let resolved = resolve_auth_sources(None, None, None, Some("jwt"), None, Some("cfg-jwt"))
        .expect("expected valid mode to resolve");

    assert_eq!(resolved.mode.label(), "jwt");
    assert_eq!(resolved.token, None);
    assert_eq!(resolved.jwt_secret.as_deref(), Some("cfg-jwt"));
}
