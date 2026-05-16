use crate::PlatformState;
use platform_contract::{AuthSummary, ConfigSummary};
use std::collections::HashMap;

/// Get config summary from state.
pub(super) fn config_summary<S>(state: &S) -> Option<ConfigSummary>
where
    S: PlatformState,
{
    let config = state.config()?;
    let auth = state.platform_auth();
    Some(ConfigSummary::new(
        config.env.clone(),
        config.http_port,
        settings_as_json(&config.settings),
        redacted_secrets(&config.secrets),
        AuthSummary::new(auth.mode_label().to_string(), auth.token_present()),
    ))
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
