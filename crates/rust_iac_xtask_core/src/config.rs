use serde::{Deserialize, Serialize};

/// Configuration for Rust IaC projects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RustIacConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub specs: SpecsConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub mode: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "rust-iac-project".to_string(),
            version: "0.1.0".to_string(),
            mode: "brownfield".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecsConfig {
    pub ledger: String,
    pub features_dir: String,
}

impl Default for SpecsConfig {
    fn default() -> Self {
        Self {
            ledger: "specs/spec_ledger.yaml".to_string(),
            features_dir: "specs/features".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub dir: String,
    pub tests_dir: String,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self { dir: "policy".to_string(), tests_dir: "policy/tests".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub contextpack: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self { contextpack: ".llm/contextpack.yaml".to_string() }
    }
}
