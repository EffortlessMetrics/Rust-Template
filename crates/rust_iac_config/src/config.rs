//! Configuration structures for IaC orchestration

use crate::error::ConfigError;
use crate::validation;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Main configuration structure for IaC orchestration
///
/// This structure defines all aspects of an IaC project including:
/// - Project metadata
/// - Deployment environments
/// - Validation rules
///
/// # Example
///
/// ```yaml
/// project:
///   name: my-project
///   workspace_root: .
///
/// environments:
///   - name: dev
///     manifests_path: infra/k8s/dev
///     requires_kustomize: false
///
/// validation:
///   check_git_repo: true
///   required_directories:
///     - specs
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IaCConfig {
    /// Project-level information
    pub project: ProjectInfo,

    /// List of deployment environments (dev, staging, prod, etc.)
    #[serde(default)]
    pub environments: Vec<Environment>,

    /// Validation rules to enforce
    #[serde(default)]
    pub validation: ValidationRules,
}

impl IaCConfig {
    /// Load and validate configuration from a YAML file
    ///
    /// This method will:
    /// 1. Check if the file exists
    /// 2. Parse the YAML content
    /// 3. Validate the configuration structure
    /// 4. Run validation checks (if enabled)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Returns
    ///
    /// Returns `Ok(IaCConfig)` if the file was successfully loaded and validated,
    /// or `Err(ConfigError)` with details about what went wrong.
    ///
    /// # Errors
    ///
    /// - [`ConfigError::FileNotFound`] if the file doesn't exist
    /// - [`ConfigError::FileReadError`] if the file can't be read
    /// - [`ConfigError::InvalidYaml`] if the YAML is malformed
    /// - Various validation errors if the configuration is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_iac_config::IaCConfig;
    /// use std::path::Path;
    ///
    /// let config = IaCConfig::from_file(Path::new("iac-config.yaml"))?;
    /// println!("Loaded config for project: {}", config.project.name);
    /// # Ok::<(), rust_iac_config::ConfigError>(())
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        // Check file exists
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }

        // Read file contents
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::file_read_error(path.to_path_buf(), e))?;

        // Parse YAML
        let config: IaCConfig =
            serde_yaml::from_str(&contents).map_err(|e| ConfigError::InvalidYaml(e.to_string()))?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate the configuration
    ///
    /// Checks:
    /// - Project name is not empty
    /// - No duplicate environment names
    /// - All required directories exist (if specified)
    /// - Git repository exists (if check_git_repo is true)
    /// - Environment configurations are valid
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate project info
        if self.project.name.trim().is_empty() {
            return Err(ConfigError::missing_field(
                "project.name",
                "Provide a non-empty project name",
            ));
        }

        // Check for duplicate environment names
        let mut seen_envs = HashSet::new();
        for env in &self.environments {
            if !seen_envs.insert(&env.name) {
                return Err(ConfigError::DuplicateEnvironment(env.name.clone()));
            }
        }

        // Validate each environment
        for env in &self.environments {
            env.validate(&self.project.workspace_root)?;
        }

        // Run validation rules
        validation::validate_config(self)?;

        Ok(())
    }

    /// Find an environment by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the environment to find (case-insensitive)
    ///
    /// # Returns
    ///
    /// Returns `Some(&Environment)` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rust_iac_config::IaCConfig;
    /// # use std::path::Path;
    /// # let config = IaCConfig::from_file(Path::new("config.yaml"))?;
    /// if let Some(env) = config.find_environment("dev") {
    ///     println!("Dev environment found at: {}", env.manifests_path.display());
    /// }
    /// # Ok::<(), rust_iac_config::ConfigError>(())
    /// ```
    pub fn find_environment(&self, name: &str) -> Option<&Environment> {
        self.environments.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Get a list of all environment names
    ///
    /// # Returns
    ///
    /// A vector of environment names as strings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rust_iac_config::IaCConfig;
    /// # use std::path::Path;
    /// # let config = IaCConfig::from_file(Path::new("config.yaml"))?;
    /// let envs = config.environment_names();
    /// println!("Available environments: {}", envs.join(", "));
    /// # Ok::<(), rust_iac_config::ConfigError>(())
    /// ```
    pub fn environment_names(&self) -> Vec<String> {
        self.environments.iter().map(|e| e.name.clone()).collect()
    }
}

/// Project-level metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectInfo {
    /// Name of the project
    pub name: String,

    /// Root directory of the workspace (relative to config file or absolute)
    #[serde(default = "default_workspace_root")]
    pub workspace_root: PathBuf,

    /// Optional description of the project
    #[serde(default)]
    pub description: Option<String>,
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from(".")
}

/// Configuration for a deployment environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    /// Name of the environment (e.g., "dev", "staging", "prod")
    pub name: String,

    /// Path to Kubernetes manifests directory for this environment
    pub manifests_path: PathBuf,

    /// Whether this environment requires Kustomize
    #[serde(default)]
    pub requires_kustomize: bool,

    /// Optional environment-specific description
    #[serde(default)]
    pub description: Option<String>,

    /// Optional custom validation rules for this environment
    #[serde(default)]
    pub required_files: Vec<String>,
}

impl Environment {
    /// Validate environment configuration
    fn validate(&self, workspace_root: &Path) -> Result<(), ConfigError> {
        // Validate environment name is not empty
        if self.name.trim().is_empty() {
            return Err(ConfigError::missing_field(
                "environment.name",
                "Each environment must have a non-empty name",
            ));
        }

        // Validate manifests path
        let manifests_full_path = workspace_root.join(&self.manifests_path);
        if !manifests_full_path.exists() {
            return Err(ConfigError::directory_not_found(
                manifests_full_path.clone(),
                format!(
                    "Create the manifests directory for the '{}' environment:\n  mkdir -p {}",
                    self.name,
                    manifests_full_path.display()
                ),
            ));
        }

        // If kustomize is required, check for kustomization.yaml
        if self.requires_kustomize {
            let kustomization_file = manifests_full_path.join("kustomization.yaml");
            if !kustomization_file.exists() {
                return Err(ConfigError::ValidationFailed(format!(
                    "Environment '{}' requires Kustomize but {} does not exist.\n\n\
                    Create a kustomization.yaml file:\n  \
                    cd {} && kustomize create --autodetect",
                    self.name,
                    kustomization_file.display(),
                    manifests_full_path.display()
                )));
            }
        }

        // Check required files
        for file in &self.required_files {
            let file_path = manifests_full_path.join(file);
            if !file_path.exists() {
                return Err(ConfigError::ValidationFailed(format!(
                    "Environment '{}' requires file '{}' but it does not exist at: {}",
                    self.name,
                    file,
                    file_path.display()
                )));
            }
        }

        Ok(())
    }
}

/// Validation rules to enforce on the configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ValidationRules {
    /// Whether to verify the project is in a git repository
    pub check_git_repo: bool,

    /// List of directories that must exist in the workspace
    pub required_directories: Vec<PathBuf>,

    /// List of files that must exist in the workspace
    pub required_files: Vec<PathBuf>,

    /// Whether to validate that manifests paths are accessible
    pub validate_manifests_paths: bool,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            check_git_repo: false,
            required_directories: Vec::new(),
            required_files: Vec::new(),
            validate_manifests_paths: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_info_default_workspace_root() {
        let yaml = r#"
name: test-project
"#;
        let info: ProjectInfo = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(info.workspace_root, PathBuf::from("."));
    }

    #[test]
    fn test_environment_name_cannot_be_empty() {
        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: PathBuf::from("."),
                description: None,
            },
            environments: vec![Environment {
                name: "".to_string(),
                manifests_path: PathBuf::from("infra/k8s/dev"),
                requires_kustomize: false,
                description: None,
                required_files: vec![],
            }],
            validation: ValidationRules::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::MissingField { field, .. }) = result {
            assert_eq!(field, "environment.name");
        } else {
            panic!("Expected MissingField error");
        }
    }

    #[test]
    fn test_duplicate_environment_names() {
        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: PathBuf::from("."),
                description: None,
            },
            environments: vec![
                Environment {
                    name: "dev".to_string(),
                    manifests_path: PathBuf::from("infra/k8s/dev"),
                    requires_kustomize: false,
                    description: None,
                    required_files: vec![],
                },
                Environment {
                    name: "dev".to_string(),
                    manifests_path: PathBuf::from("infra/k8s/dev2"),
                    requires_kustomize: false,
                    description: None,
                    required_files: vec![],
                },
            ],
            validation: ValidationRules::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        if let Err(ConfigError::DuplicateEnvironment(name)) = result {
            assert_eq!(name, "dev");
        } else {
            panic!("Expected DuplicateEnvironment error");
        }
    }

    #[test]
    fn test_find_environment_case_insensitive() {
        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: PathBuf::from("."),
                description: None,
            },
            environments: vec![Environment {
                name: "Development".to_string(),
                manifests_path: PathBuf::from("infra/k8s/dev"),
                requires_kustomize: false,
                description: None,
                required_files: vec![],
            }],
            validation: ValidationRules::default(),
        };

        assert!(config.find_environment("development").is_some());
        assert!(config.find_environment("DEVELOPMENT").is_some());
        assert!(config.find_environment("Development").is_some());
        assert!(config.find_environment("staging").is_none());
    }

    #[test]
    fn test_environment_names() {
        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: PathBuf::from("."),
                description: None,
            },
            environments: vec![
                Environment {
                    name: "dev".to_string(),
                    manifests_path: PathBuf::from("infra/k8s/dev"),
                    requires_kustomize: false,
                    description: None,
                    required_files: vec![],
                },
                Environment {
                    name: "staging".to_string(),
                    manifests_path: PathBuf::from("infra/k8s/staging"),
                    requires_kustomize: true,
                    description: None,
                    required_files: vec![],
                },
            ],
            validation: ValidationRules::default(),
        };

        let names = config.environment_names();
        assert_eq!(names, vec!["dev", "staging"]);
    }
}
