use http_platform_status_summary::ConfigSummary;
use serde_yaml::Value as YamlValue;
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;

#[test]
fn integration_from_parts_emits_redacted_secrets_for_platform_status_payload() {
    let mut settings = HashMap::new();
    settings.insert("platform.auth_mode".to_string(), YamlValue::String("jwt".to_string()));

    let mut secrets = HashMap::new();
    secrets.insert("platform.auth_token".to_string(), "super-secret-token".to_string());
    secrets.insert("platform.jwt_secret".to_string(), "super-secret-jwt".to_string());

    let config =
        ValidatedConfig { env: Some("test".to_string()), http_port: 8080, settings, secrets };

    let summary = ConfigSummary::from_parts(&config, "jwt", true);

    let body = serde_json::to_string(&summary).expect("summary should serialize");
    assert!(!body.contains("super-secret-token"));
    assert!(!body.contains("super-secret-jwt"));
    assert!(body.contains("\"mode\":\"jwt\""));
    assert!(body.contains("\"token_present\":true"));
}
