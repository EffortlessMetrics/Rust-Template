use http_auth_policy::{PlatformAuthConfig, PlatformAuthMode};

#[test]
fn integration_basic_mode_accepts_expected_token_and_rejects_other_values() {
    let config = PlatformAuthConfig {
        mode: PlatformAuthMode::Basic,
        token: Some("integration-token".to_string()),
        jwt_secret: Some("unused-secret".to_string()),
    };

    assert!(config.is_authorized(Some("integration-token")));
    assert!(!config.is_authorized(Some("other-token")));
    assert!(!config.is_authorized(None));
}

#[test]
fn integration_open_mode_is_authorized_with_any_input() {
    let config = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None, jwt_secret: None };

    assert!(config.is_authorized(None));
    assert!(config.is_authorized(Some("anything")));
}
