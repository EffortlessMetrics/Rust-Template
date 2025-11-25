use anyhow::{Result, anyhow};
use serde_yaml::Value;
use std::fs;

/// Minimal config-validate: parse `specs/config_schema.yaml` and perform basic checks.
pub fn run(env: &str) -> Result<()> {
    let text = fs::read_to_string("specs/config_schema.yaml")
        .map_err(|e| anyhow!("failed to read config schema: {}", e))?;

    let doc: Value = serde_yaml::from_str(&text)?;

    // Basic validations
    if doc.get("settings").is_none() && doc.get("secrets").is_none() {
        anyhow::bail!("config schema must contain at least one of 'settings' or 'secrets'");
    }

    println!("Parsed config schema for env='{}' successfully", env);
    if let Some(Value::Sequence(secrets)) = doc.get("secrets") {
        for s in secrets.iter() {
            if let Value::Mapping(m) = s
                && !m.contains_key(Value::from("key"))
            {
                anyhow::bail!("each secret entry must have a 'key' field");
            }
        }
    }

    println!("Basic schema checks passed");
    println!("Note: this is a lightweight validator; environment resolution not implemented yet.");

    Ok(())
}
