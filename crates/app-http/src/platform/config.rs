use crate::AppState;
use serde::Serialize;
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub(crate) struct ConfigSummary {
    env: Option<String>,
    http_port: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    settings: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    secrets_redacted: HashMap<String, String>,
    auth: AuthSummary,
}

#[derive(Serialize, Clone)]
struct AuthSummary {
    mode: String,
    token_present: bool,
}

pub(crate) fn config_summary(state: &AppState) -> Option<ConfigSummary> {
    let config = state.config.as_ref()?;
    Some(ConfigSummary::from_parts(config, &state.platform_auth))
}

impl ConfigSummary {
    fn from_parts(config: &ValidatedConfig, auth: &crate::security::PlatformAuthConfig) -> Self {
        let settings = settings_as_json(&config.settings);

        ConfigSummary {
            env: config.env.clone(),
            http_port: config.http_port,
            settings,
            secrets_redacted: redacted_secrets(&config.secrets),
            auth: AuthSummary {
                mode: auth.mode_label().to_string(),
                token_present: auth.token_present(),
            },
        }
    }
}

fn settings_as_json(
    source: &HashMap<String, serde_yaml::Value>,
) -> HashMap<String, serde_json::Value> {
    let mut out = HashMap::new();

    for (k, v) in source {
        if let Ok(json_val) = serde_json::to_value(v) {
            out.insert(k.clone(), json_val);
        }
    }

    out
}

fn redacted_secrets(secrets: &HashMap<String, String>) -> HashMap<String, String> {
    secrets.keys().map(|k| (k.clone(), "[REDACTED]".to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{PlatformAuthConfig, PlatformAuthMode};

    #[test]
    fn log_hygiene_redacts_secrets() {
        let mut settings = HashMap::new();
        settings
            .insert("platform.auth_mode".to_string(), serde_yaml::Value::String("basic".into()));

        let mut secrets = HashMap::new();
        secrets.insert("db.url".to_string(), "postgres://user:pass@localhost:5432/app".to_string());
        secrets.insert("platform.auth_token".to_string(), "super-secret-token".to_string());

        let config =
            ValidatedConfig { http_port: 8080, env: Some("dev".to_string()), settings, secrets };

        let auth = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("super-secret-token".into()),
            jwt_secret: None,
        };
        let summary = ConfigSummary::from_parts(&config, &auth);

        let serialized = serde_json::to_string(&summary).expect("summary should serialize");

        assert!(
            !serialized.contains("super-secret-token"),
            "Serialized summary should not leak auth tokens"
        );
        assert_eq!(summary.secrets_redacted.get("db.url"), Some(&"[REDACTED]".to_string()));
        assert_eq!(summary.auth.mode, "basic");
        assert!(summary.auth.token_present);
    }
}
