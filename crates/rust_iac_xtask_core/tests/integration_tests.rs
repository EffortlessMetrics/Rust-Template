use rust_iac_xtask_core::{ConfigError, IaCConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_load_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Create required directories
    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: dev
    manifests_path: infra/k8s/dev
    requires_kustomize: false

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    // Change to temp directory so relative paths work
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.project.name, "test-project");
    assert_eq!(config.environments.len(), 1);
    assert_eq!(config.environments[0].name, "dev");
}

#[test]
fn test_file_not_found_error() {
    let result = IaCConfig::from_file(&PathBuf::from("/nonexistent/config.yaml"));

    assert!(result.is_err());
    match result {
        Err(ConfigError::FileNotFound(path)) => {
            assert_eq!(path, PathBuf::from("/nonexistent/config.yaml"));
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_invalid_yaml_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let invalid_yaml = r#"
project:
  name: test
  this is not valid yaml ][}{
"#;

    fs::write(&config_path, invalid_yaml).unwrap();

    let result = IaCConfig::from_file(&config_path);

    assert!(result.is_err());
    match result {
        Err(ConfigError::InvalidYaml(_)) => {}
        _ => panic!("Expected InvalidYaml error"),
    }
}

#[test]
fn test_missing_required_field() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Missing project.name
    let config_content = r#"
project:
  name: ""
  workspace_root: .
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = IaCConfig::from_file(&config_path);

    assert!(result.is_err());
    match result {
        Err(ConfigError::MissingField { field, .. }) => {
            assert_eq!(field, "project.name");
        }
        _ => panic!("Expected MissingField error"),
    }
}

#[test]
fn test_duplicate_environment_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev")).unwrap();
    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev2")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: dev
    manifests_path: infra/k8s/dev
  - name: dev
    manifests_path: infra/k8s/dev2

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
    match result {
        Err(ConfigError::DuplicateEnvironment(name)) => {
            assert_eq!(name, "dev");
        }
        _ => panic!("Expected DuplicateEnvironment error"),
    }
}

#[test]
fn test_manifests_directory_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Don't create the manifests directory
    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: dev
    manifests_path: infra/k8s/dev

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
    match result {
        Err(ConfigError::DirectoryNotFound { path, hint }) => {
            assert!(path.to_string_lossy().contains("infra/k8s/dev"));
            assert!(hint.contains("mkdir"));
        }
        _ => panic!("Expected DirectoryNotFound error"),
    }
}

#[test]
fn test_kustomize_required_but_missing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    fs::create_dir_all(temp_dir.path().join("infra/k8s/staging")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: staging
    manifests_path: infra/k8s/staging
    requires_kustomize: true

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
    match result {
        Err(ConfigError::ValidationFailed(msg)) => {
            assert!(msg.contains("kustomization.yaml"));
            assert!(msg.contains("staging"));
        }
        _ => panic!("Expected ValidationFailed error for missing kustomization.yaml"),
    }
}

#[test]
fn test_find_environment() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev")).unwrap();
    fs::create_dir_all(temp_dir.path().join("infra/k8s/prod")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: dev
    manifests_path: infra/k8s/dev
  - name: prod
    manifests_path: infra/k8s/prod

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = IaCConfig::from_file(&PathBuf::from("config.yaml")).unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    // Test case-insensitive lookup
    assert!(config.find_environment("dev").is_some());
    assert!(config.find_environment("DEV").is_some());
    assert!(config.find_environment("Dev").is_some());
    assert!(config.find_environment("prod").is_some());
    assert!(config.find_environment("staging").is_none());

    // Verify the found environment has correct properties
    let dev_env = config.find_environment("dev").unwrap();
    assert_eq!(dev_env.name, "dev");
    assert_eq!(dev_env.manifests_path, PathBuf::from("infra/k8s/dev"));
}

#[test]
fn test_required_directories_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Create one directory but not the other
    fs::create_dir_all(temp_dir.path().join("specs")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

validation:
  check_git_repo: false
  required_directories:
    - specs
    - infra
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
    match result {
        Err(ConfigError::DirectoryNotFound { path, .. }) => {
            assert!(path.to_string_lossy().contains("infra"));
        }
        _ => panic!("Expected DirectoryNotFound error"),
    }
}

#[test]
fn test_required_files_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

validation:
  check_git_repo: false
  required_files:
    - README.md
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = IaCConfig::from_file(&PathBuf::from("config.yaml"));

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
    match result {
        Err(ConfigError::ValidationFailed(msg)) => {
            assert!(msg.contains("README.md"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_environment_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev")).unwrap();
    fs::create_dir_all(temp_dir.path().join("infra/k8s/staging")).unwrap();
    fs::create_dir_all(temp_dir.path().join("infra/k8s/prod")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .

environments:
  - name: dev
    manifests_path: infra/k8s/dev
  - name: staging
    manifests_path: infra/k8s/staging
  - name: prod
    manifests_path: infra/k8s/prod

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = IaCConfig::from_file(&PathBuf::from("config.yaml")).unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    let names = config.environment_names();
    assert_eq!(names, vec!["dev", "staging", "prod"]);
}

#[test]
fn test_config_with_descriptions() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    fs::create_dir_all(temp_dir.path().join("infra/k8s/dev")).unwrap();

    let config_content = r#"
project:
  name: test-project
  workspace_root: .
  description: A test project for IaC orchestration

environments:
  - name: dev
    manifests_path: infra/k8s/dev
    description: Local development environment

validation:
  check_git_repo: false
"#;

    fs::write(&config_path, config_content).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let config = IaCConfig::from_file(&PathBuf::from("config.yaml")).unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(
        config.project.description,
        Some("A test project for IaC orchestration".to_string())
    );
    assert_eq!(
        config.environments[0].description,
        Some("Local development environment".to_string())
    );
}
