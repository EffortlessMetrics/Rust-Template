//! Configuration schema validation and runtime config management.
//!
//! This module provides schema-driven configuration validation, ensuring that config files
//! conform to a defined schema before being used by the application.

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A validated configuration instance with typed fields and merged settings/secrets.
///
/// This is the result of validating a config file against a schema. All required
/// fields have been checked, types validated, and defaults applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedConfig {
    /// Optional environment identifier (e.g., "local", "dev", "prod").
    pub env: Option<String>,
    /// HTTP server port (defaults to 8080 if not specified).
    pub http_port: u16,
    /// Application settings as key-value pairs (merged from schema + config).
    pub settings: HashMap<String, Value>,
    /// Application secrets as string key-value pairs.
    pub secrets: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ConfigSchema {
    #[serde(default)]
    envs: Vec<EnvSpec>,
    #[serde(default)]
    settings: Vec<SchemaEntry>,
    #[serde(default)]
    secrets: Vec<SchemaEntry>,
}

#[derive(Debug, Deserialize)]
struct EnvSpec {
    name: String,
    #[serde(default)]
    required: bool,
}

#[derive(Debug, Deserialize)]
struct SchemaEntry {
    key: String,
    #[serde(rename = "type")]
    value_type: String,
    #[serde(default = "required_true")]
    required: bool,
    #[serde(default)]
    default: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    env: Option<String>,
    #[serde(default)]
    settings: HashMap<String, Value>,
    #[serde(default)]
    secrets: HashMap<String, Value>,
}

fn required_true() -> bool {
    true
}

/// Validate a configuration file against a schema.
///
/// This function reads both the schema and config files, validates that all required
/// fields are present, checks types, applies defaults, and returns a merged [`ValidatedConfig`].
///
/// # Arguments
///
/// * `schema_path` - Path to the config schema YAML file (e.g., `specs/config_schema.yaml`)
/// * `config_path` - Path to the config YAML file (e.g., `config/local.yaml`)
///
/// # Returns
///
/// Returns a [`ValidatedConfig`] with all settings and secrets validated and merged.
///
/// # Errors
///
/// Returns an error if:
/// - Either file is missing or malformed YAML
/// - Required fields are missing
/// - Field types don't match the schema
/// - Environment validation fails
///
/// # Example
///
/// ```ignore
/// let config = validate_config(
///     Path::new("specs/config_schema.yaml"),
///     Path::new("config/local.yaml")
/// )?;
/// println!("HTTP port: {}", config.http_port);
/// ```
pub fn validate_config(schema_path: &Path, config_path: &Path) -> Result<ValidatedConfig> {
    if !schema_path.exists() {
        bail!("Config schema not found at {}", schema_path.display());
    }

    if !config_path.exists() {
        bail!("Config file not found at {}", config_path.display());
    }

    let schema: ConfigSchema = serde_yaml::from_str(
        &fs::read_to_string(schema_path)
            .with_context(|| format!("Failed to read {}", schema_path.display()))?,
    )
    .with_context(|| format!("Failed to parse {}", schema_path.display()))?;

    let config: ConfigFile = serde_yaml::from_str(
        &fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?,
    )
    .with_context(|| format!("Failed to parse {}", config_path.display()))?;

    validate_env(&schema.envs, config.env.as_deref())?;

    let mut merged_settings = HashMap::new();
    let mut http_port: Option<u16> = None;

    for entry in &schema.settings {
        let value = config.settings.get(&entry.key).cloned().or_else(|| entry.default.clone());
        let value = match value {
            Some(v) => v,
            None if entry.required => bail!("Missing required setting '{}'", entry.key),
            None => continue,
        };

        validate_type(&entry.value_type, &value)
            .with_context(|| format!("Invalid type for setting '{}'", entry.key))?;

        if entry.key == "http.port" {
            let port_val = value.as_i64().ok_or_else(|| anyhow!("http.port must be an integer"))?;
            http_port = Some(
                u16::try_from(port_val)
                    .map_err(|_| anyhow!("http.port must be between 0 and 65535"))?,
            );
        }

        merged_settings.insert(entry.key.clone(), value);
    }

    let mut merged_secrets = HashMap::new();
    for entry in &schema.secrets {
        let value = config.secrets.get(&entry.key).cloned().or_else(|| entry.default.clone());

        let value = match value {
            Some(v) => v,
            None if entry.required => bail!("Missing required secret '{}'", entry.key),
            None => continue,
        };
        validate_type(&entry.value_type, &value)
            .with_context(|| format!("Invalid type for secret '{}'", entry.key))?;

        let value_str =
            value.as_str().ok_or_else(|| anyhow!("Secret '{}' must be a string", entry.key))?;
        merged_secrets.insert(entry.key.clone(), value_str.to_string());
    }

    Ok(ValidatedConfig {
        env: config.env.clone(),
        http_port: http_port.unwrap_or(8080),
        settings: merged_settings,
        secrets: merged_secrets,
    })
}

fn validate_env(envs: &[EnvSpec], value: Option<&str>) -> Result<()> {
    if envs.is_empty() {
        return Ok(());
    }

    if let Some(name) = value {
        if let Some(spec) = envs.iter().find(|e| e.name == name) {
            if !spec.required {
                return Ok(());
            }
        } else {
            bail!("Environment '{}' is not defined in config schema", name);
        }
    } else if envs.iter().any(|e| e.required) {
        bail!("Environment not specified and at least one env is required");
    }

    Ok(())
}

fn validate_type(expected: &str, value: &Value) -> Result<()> {
    match expected {
        "int" => {
            if value.as_i64().is_none() {
                bail!("Expected int, got {:?}", value);
            }
        }
        "string" => {
            if value.as_str().is_none() {
                bail!("Expected string, got {:?}", value);
            }
        }
        "bool" => {
            if value.as_bool().is_none() {
                bail!("Expected bool, got {:?}", value);
            }
        }
        other => bail!("Unsupported type '{}' in schema", other),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn config_validation_rejects_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let schema_path = dir.path().join("schema.yaml");
        let config_path = dir.path().join("config.yaml");

        fs::write(
            &schema_path,
            r#"
settings:
  - key: http.port
    type: int
    required: true
secrets:
  - key: db.url
    type: string
    required: true
"#,
        )
        .unwrap();

        // Missing db.url secret
        let mut config = fs::File::create(&config_path).unwrap();
        writeln!(
            config,
            r#"
settings:
  http.port: 8080
secrets: {{}}
"#
        )
        .unwrap();

        let result = validate_config(&schema_path, &config_path);
        assert!(
            result.is_err(),
            "Expected config validation to fail when required secrets are missing"
        );
    }
}
