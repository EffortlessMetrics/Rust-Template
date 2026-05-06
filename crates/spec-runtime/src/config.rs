//! Configuration schema validation and runtime config types.

use crate::error::{Result, SpecError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Validated runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedConfig {
    /// Environment name.
    pub env: Option<String>,
    /// HTTP port to listen on.
    pub http_port: u16,
    /// Service settings.
    pub settings: HashMap<String, serde_yaml::Value>,
    /// Redacted secrets.
    pub secrets: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ConfigSchema {
    /// Environment definitions (reserved for future multi-env support).
    #[expect(dead_code, reason = "existing reviewed debt; tracked by lint policy ratchet")]
    pub envs: Vec<EnvDef>,
    pub settings: Vec<ConfigEntry>,
    pub secrets: Vec<ConfigEntry>,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "existing reviewed debt; tracked by lint policy ratchet")]
struct EnvDef {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
struct ConfigEntry {
    pub key: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub default: Option<serde_yaml::Value>,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Validate a configuration file against a schema.
pub fn validate_config(schema_path: &Path, config_path: &Path) -> Result<ValidatedConfig> {
    let schema_content =
        std::fs::read_to_string(schema_path).map_err(|e| SpecError::io(schema_path, e))?;
    let schema: ConfigSchema = serde_yaml::from_str(&schema_content).map_err(SpecError::Yaml)?;

    let config_content =
        std::fs::read_to_string(config_path).map_err(|e| SpecError::io(config_path, e))?;
    let config: serde_yaml::Value =
        serde_yaml::from_str(&config_content).map_err(SpecError::Yaml)?;

    let config_map = config
        .as_mapping()
        .ok_or_else(|| SpecError::ConfigValidation("Config must be a YAML mapping".to_string()))?;

    let config_settings = config_map
        .get(serde_yaml::Value::String("settings".to_string()))
        .and_then(|v| v.as_mapping());
    let config_secrets = config_map
        .get(serde_yaml::Value::String("secrets".to_string()))
        .and_then(|v| v.as_mapping());

    let mut settings = HashMap::new();
    let mut secrets = HashMap::new();

    // Process settings
    for entry in &schema.settings {
        let value = config_settings
            .and_then(|m| m.get(serde_yaml::Value::String(entry.key.clone())))
            .cloned()
            .or_else(|| entry.default.clone());

        match value {
            Some(v) => {
                // Type validation (basic)
                validate_type(&entry.key, &v, &entry.entry_type)?;
                settings.insert(entry.key.clone(), v);
            }
            None if entry.required => {
                return Err(SpecError::ConfigValidation(format!(
                    "Missing required setting: {}",
                    entry.key
                )));
            }
            None => {}
        }
    }

    // Process secrets
    for entry in &schema.secrets {
        let value = config_secrets
            .and_then(|m| m.get(serde_yaml::Value::String(entry.key.clone())))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match value {
            Some(v) => {
                secrets.insert(entry.key.clone(), v);
            }
            None if entry.required => {
                return Err(SpecError::ConfigValidation(format!(
                    "Missing required secret: {}",
                    entry.key
                )));
            }
            None => {}
        }
    }

    let http_port =
        settings.get("http.port").and_then(|v| v.as_u64()).map(|v| v as u16).unwrap_or(8080);

    Ok(ValidatedConfig { env: Some("local".to_string()), http_port, settings, secrets })
}

fn validate_type(key: &str, value: &serde_yaml::Value, expected_type: &str) -> Result<()> {
    let valid = match expected_type {
        "int" => value.is_u64() || value.is_i64(),
        "string" => value.is_string(),
        "bool" => value.is_bool(),
        _ => true, // Unknown type, skip validation
    };

    if !valid {
        return Err(SpecError::ConfigValidation(format!(
            "Type mismatch for {}: expected {}, found {:?}",
            key, expected_type, value
        )));
    }
    Ok(())
}
