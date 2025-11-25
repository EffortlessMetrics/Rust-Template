use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub service_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub template_version: Option<String>,
    pub ownership: Ownership,
    pub lifecycle: Lifecycle,
    #[serde(default)]
    pub links: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub permissions: Option<Permissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ownership {
    pub team: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub slack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub tier: u8,
    pub data_class: String,
    pub criticality: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub runtime: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub default_roles: Vec<String>,
}

pub fn load_service_metadata(path: &Path) -> Result<ServiceMetadata> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let meta = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_metadata() {
        let yaml = r#"
service_id: rust-template
display_name: "Rust-as-Spec Platform Cell"
description: >
  Governed Rust service template.

ownership:
  team: plat-governance
  email: platform-governance@example.com

lifecycle:
  tier: 1
  data_class: internal
  criticality: platform
  languages: [rust]
  runtime: [kubernetes]

links:
  docs: "https://.../docs"

tags:
  - rust
  - governance
"#;
        let meta: ServiceMetadata = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(meta.service_id, "rust-template");
        assert_eq!(meta.ownership.team, "plat-governance");
        assert_eq!(meta.lifecycle.tier, 1);
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn load_service_metadata_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("service_metadata.yaml");
        std::fs::write(
            &path,
            r#"
service_id: test-service
ownership:
  team: test
lifecycle:
  tier: 1
  data_class: internal
  criticality: low
"#,
        )
        .unwrap();

        let meta = load_service_metadata(&path).unwrap();
        assert_eq!(meta.service_id, "test-service");
    }
}
