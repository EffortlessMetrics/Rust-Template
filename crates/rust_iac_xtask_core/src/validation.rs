//! Validation logic for IaC configurations

use crate::config::IaCConfig;
use crate::error::ConfigError;
use std::path::Path;
use std::process::Command;

/// Validate the entire configuration based on validation rules
pub(crate) fn validate_config(config: &IaCConfig) -> Result<(), ConfigError> {
    let workspace_root = &config.project.workspace_root;

    // Check if git repository is required
    if config.validation.check_git_repo {
        validate_git_repository(workspace_root)?;
    }

    // Check required directories
    for dir in &config.validation.required_directories {
        let dir_path = workspace_root.join(dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(ConfigError::directory_not_found(
                dir_path.clone(),
                format!(
                    "This directory is required by the project configuration.\n\
                    Create it with:\n  mkdir -p {}",
                    dir_path.display()
                ),
            ));
        }
    }

    // Check required files
    for file in &config.validation.required_files {
        let file_path = workspace_root.join(file);
        if !file_path.exists() || !file_path.is_file() {
            return Err(ConfigError::ValidationFailed(format!(
                "Required file not found: {}\n\n\
                This file is required by the project configuration.",
                file_path.display()
            )));
        }
    }

    Ok(())
}

/// Validate that the workspace is in a git repository
fn validate_git_repository(workspace_root: &Path) -> Result<(), ConfigError> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(workspace_root)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(ConfigError::NotGitRepository),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProjectInfo, ValidationRules};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_validate_required_directories() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        // Create one required directory but not the other
        let specs_dir = workspace_root.join("specs");
        std::fs::create_dir(&specs_dir).unwrap();

        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: workspace_root.to_path_buf(),
                description: None,
            },
            environments: vec![],
            validation: ValidationRules {
                check_git_repo: false,
                required_directories: vec![PathBuf::from("specs"), PathBuf::from("infra")],
                required_files: vec![],
                validate_manifests_paths: false,
            },
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(ConfigError::DirectoryNotFound { path, .. }) = result {
            assert!(path.ends_with("infra"));
        } else {
            panic!("Expected DirectoryNotFound error");
        }
    }

    #[test]
    fn test_validate_required_files() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: workspace_root.to_path_buf(),
                description: None,
            },
            environments: vec![],
            validation: ValidationRules {
                check_git_repo: false,
                required_directories: vec![],
                required_files: vec![PathBuf::from("README.md")],
                validate_manifests_paths: false,
            },
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(ConfigError::ValidationFailed(msg)) = result {
            assert!(msg.contains("README.md"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_validate_all_requirements_met() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        // Create required directory and file
        let specs_dir = workspace_root.join("specs");
        std::fs::create_dir(&specs_dir).unwrap();

        let readme_path = workspace_root.join("README.md");
        std::fs::write(&readme_path, "# Test Project").unwrap();

        let config = IaCConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                workspace_root: workspace_root.to_path_buf(),
                description: None,
            },
            environments: vec![],
            validation: ValidationRules {
                check_git_repo: false,
                required_directories: vec![PathBuf::from("specs")],
                required_files: vec![PathBuf::from("README.md")],
                validate_manifests_paths: false,
            },
        };

        let result = validate_config(&config);
        assert!(result.is_ok());
    }
}
