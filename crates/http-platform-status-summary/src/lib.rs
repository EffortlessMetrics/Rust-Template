#![forbid(unsafe_code)]

//! Status summary payload modeling for platform responses.

use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;

/// Summary information exposed by `/platform/status`.
#[derive(Clone, Debug, Serialize)]
pub struct ConfigSummary {
    /// Environment name associated with the validated config, if present.
    pub env: Option<String>,
    /// Configured HTTP port.
    pub http_port: u16,
    /// Settings rendered as JSON for stable status payload serialization.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub settings: HashMap<String, JsonValue>,
    /// Secrets map with values masked to avoid leaking sensitive data.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub secrets_redacted: HashMap<String, String>,
    /// Authentication mode and readiness for status output.
    pub auth: AuthSummary,
}

#[derive(Clone, Debug, Serialize)]
/// Authentication summary exposed from `/platform/status`.
pub struct AuthSummary {
    /// Serialized auth mode label for status payloads.
    pub mode: String,
    /// Whether a token credential is currently configured.
    pub token_present: bool,
}

impl ConfigSummary {
    /// Build a status summary from validated config and auth policy state.
    pub fn from_parts(config: &ValidatedConfig, mode_label: &str, token_present: bool) -> Self {
        Self {
            env: config.env.clone(),
            http_port: config.http_port,
            settings: settings_as_json(&config.settings),
            secrets_redacted: redacted_secrets(&config.secrets),
            auth: AuthSummary { mode: mode_label.to_string(), token_present },
        }
    }
}

fn settings_as_json(source: &HashMap<String, YamlValue>) -> HashMap<String, JsonValue> {
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
    use proptest::prelude::*;

    fn key() -> impl Strategy<Value = String> {
        "[a-z0-9._-]{1,12}".prop_map(|value| value.to_string())
    }

    fn non_empty_key() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9._-]{1,16}".prop_map(|value| value.to_string())
    }

    #[test]
    fn from_parts_masks_secrets_and_preserves_metadata() {
        let config = ValidatedConfig {
            env: Some("test".to_string()),
            http_port: 8080,
            settings: [("platform.auth_mode".to_string(), YamlValue::String("basic".to_string()))]
                .into_iter()
                .collect(),
            secrets: [
                ("platform.auth_token".to_string(), "secret-token".to_string()),
                ("platform.jwt_secret".to_string(), "secret-jwt".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let summary = ConfigSummary::from_parts(&config, "basic", true);

        assert_eq!(summary.env.as_deref(), Some("test"));
        assert_eq!(summary.http_port, 8080);
        assert_eq!(
            summary.settings.get("platform.auth_mode"),
            Some(&JsonValue::String("basic".into()))
        );
        assert_eq!(summary.secrets_redacted.len(), 2);
        assert_eq!(
            summary.secrets_redacted.get("platform.auth_token"),
            Some(&"[REDACTED]".to_string())
        );
        assert_eq!(summary.auth.mode, "basic");
        assert!(summary.auth.token_present);
    }

    #[test]
    fn redaction_is_total_for_all_secret_keys() {
        let secrets = [("a".to_string(), "one".to_string()), ("b".to_string(), "two".to_string())]
            .into_iter()
            .collect();

        assert_eq!(
            redacted_secrets(&secrets),
            [
                ("a".to_string(), "[REDACTED]".to_string()),
                ("b".to_string(), "[REDACTED]".to_string())
            ]
            .into_iter()
            .collect()
        );
    }

    mod proptests {
        use super::*;
        use std::collections::HashMap;

        fn env_name() -> impl Strategy<Value = String> {
            prop_oneof![Just("test".to_string()), Just("ci".to_string()), Just("prod".to_string()),]
        }

        fn mode_strategy() -> impl Strategy<Value = String> {
            prop_oneof![
                Just("open".to_string()),
                Just("basic".to_string()),
                Just("jwt".to_string()),
                Just("none".to_string()),
                any::<String>(),
            ]
        }

        proptest! {
            #[test]
            fn prop_from_parts_redacts_every_secret(
                env in proptest::option::of(env_name()),
                http_port in any::<u16>(),
                settings in prop::collection::hash_map(key(), key(), 0..10),
                secrets in prop::collection::hash_map(non_empty_key(), key(), 0..10),
                token_present in any::<bool>(),
                mode in mode_strategy(),
            ) {
                let settings = settings
                    .into_iter()
                    .map(|(k, v)| (k, YamlValue::String(v)))
                    .collect::<HashMap<String, YamlValue>>();

                let cfg = ValidatedConfig {
                    env,
                    http_port,
                    settings,
                    secrets,
                };

                let summary = ConfigSummary::from_parts(&cfg, &mode, token_present);

                prop_assert_eq!(summary.http_port, cfg.http_port);
                prop_assert_eq!(summary.env, cfg.env);
                prop_assert_eq!(summary.auth.mode, mode);
                prop_assert_eq!(summary.auth.token_present, token_present);
                prop_assert_eq!(summary.secrets_redacted.len(), cfg.secrets.len());
                prop_assert!(summary.secrets_redacted.values().all(|value| value == "[REDACTED]"));
            }
        }
    }
}
