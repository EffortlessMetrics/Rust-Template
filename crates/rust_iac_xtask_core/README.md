# rust_iac_xtask_core

Production-ready Infrastructure as Code (IaC) orchestration library for Rust projects.

## Overview

`rust_iac_xtask_core` provides a configuration-driven framework for orchestrating development and deployment workflows in Rust-based IaC templates. It is designed to be reusable across different projects and makes no assumptions about specific acceptance criteria IDs, environment names, or project structure beyond what is explicitly configured.

## Features

- **Configuration-driven**: Define environments, validation rules, and project structure in YAML
- **Strong validation**: Comprehensive validation with clear, actionable error messages
- **No panics**: All error conditions return `Result` types with structured errors
- **Production-ready**: Designed for external users with a clean, minimal API
- **Zero hardcoded assumptions**: Works with any project structure defined in config

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust_iac_xtask_core = "0.1.0"
```

## Quick Start

### 1. Create a configuration file

Create `iac-config.yaml` in your project root:

```yaml
project:
  name: my-rust-project
  workspace_root: .
  description: My awesome Rust IaC project

environments:
  - name: dev
    manifests_path: infra/k8s/dev
    requires_kustomize: false
    description: Local development environment

  - name: staging
    manifests_path: infra/k8s/staging
    requires_kustomize: true
    description: Staging environment with Kustomize

  - name: prod
    manifests_path: infra/k8s/prod
    requires_kustomize: true
    description: Production environment

validation:
  check_git_repo: true
  required_directories:
    - specs
    - infra
  required_files:
    - README.md
```

### 2. Load and use the configuration

```rust
use rust_iac_xtask_core::{IaCConfig, ConfigError};
use std::path::Path;

fn main() -> Result<(), ConfigError> {
    // Load configuration
    let config = IaCConfig::from_file(Path::new("iac-config.yaml"))?;

    println!("Project: {}", config.project.name);
    println!("Available environments: {}", config.environment_names().join(", "));

    // Find a specific environment
    if let Some(env) = config.find_environment("dev") {
        println!("Dev manifests: {}", env.manifests_path.display());
    }

    Ok(())
}
```

## Configuration Reference

### Project Section

The `project` section defines project-level metadata:

```yaml
project:
  name: my-project              # Required: Project name
  workspace_root: .              # Optional: Path to workspace root (default: ".")
  description: My project        # Optional: Project description
```

### Environments Section

The `environments` section defines deployment targets:

```yaml
environments:
  - name: dev                    # Required: Environment name
    manifests_path: infra/k8s/dev  # Required: Path to K8s manifests
    requires_kustomize: false    # Optional: Whether kustomization.yaml is required
    description: Dev env         # Optional: Environment description
    required_files:              # Optional: Files that must exist
      - deployment.yaml
      - service.yaml
```

### Validation Section

The `validation` section defines validation rules:

```yaml
validation:
  check_git_repo: true          # Optional: Verify project is in git repo
  required_directories:         # Optional: Directories that must exist
    - specs
    - infra
  required_files:               # Optional: Files that must exist
    - README.md
  validate_manifests_paths: true  # Optional: Validate manifests paths exist
```

## Error Handling

All errors are represented by the `ConfigError` type:

```rust
use rust_iac_xtask_core::{IaCConfig, ConfigError};
use std::path::Path;

match IaCConfig::from_file(Path::new("config.yaml")) {
    Ok(config) => {
        println!("Config loaded successfully");
    }
    Err(ConfigError::FileNotFound(path)) => {
        eprintln!("Config file not found: {}", path.display());
    }
    Err(ConfigError::InvalidYaml(msg)) => {
        eprintln!("Invalid YAML: {}", msg);
    }
    Err(ConfigError::DirectoryNotFound { path, hint }) => {
        eprintln!("Directory not found: {}", path.display());
        eprintln!("Hint: {}", hint);
    }
    Err(ConfigError::ValidationFailed(msg)) => {
        eprintln!("Validation failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Error Types

- `FileNotFound`: Configuration file doesn't exist
- `FileReadError`: Failed to read configuration file
- `InvalidYaml`: Malformed YAML syntax
- `MissingField`: Required configuration field is missing
- `InvalidValue`: Invalid value for a configuration field
- `DirectoryNotFound`: Required directory doesn't exist
- `EnvironmentNotFound`: Specified environment not in configuration
- `DuplicateEnvironment`: Duplicate environment names
- `NotGitRepository`: Not in a git repository (when check enabled)
- `ValidationFailed`: Generic validation failure

## API Reference

### `IaCConfig`

The main configuration structure.

#### Methods

- `from_file(path: &Path) -> Result<Self, ConfigError>`: Load and validate configuration from a YAML file
- `find_environment(name: &str) -> Option<&Environment>`: Find an environment by name (case-insensitive)
- `environment_names() -> Vec<String>`: Get list of all environment names

#### Fields

- `project: ProjectInfo`: Project metadata
- `environments: Vec<Environment>`: List of deployment environments
- `validation: ValidationRules`: Validation rules

### `ProjectInfo`

Project-level metadata.

- `name: String`: Project name
- `workspace_root: PathBuf`: Workspace root directory
- `description: Option<String>`: Optional description

### `Environment`

Deployment environment configuration.

- `name: String`: Environment name
- `manifests_path: PathBuf`: Path to Kubernetes manifests
- `requires_kustomize: bool`: Whether Kustomize is required
- `description: Option<String>`: Optional description
- `required_files: Vec<String>`: Files that must exist in manifests directory

### `ValidationRules`

Validation rules for the configuration.

- `check_git_repo: bool`: Verify project is in a git repository
- `required_directories: Vec<PathBuf>`: Directories that must exist
- `required_files: Vec<PathBuf>`: Files that must exist
- `validate_manifests_paths: bool`: Validate manifests paths exist

## Design Principles

This library follows these principles:

1. **No panics**: All error conditions return `Result` types
2. **Validation first**: Configuration is validated on load, not at use time
3. **Clear errors**: Error messages explain what's wrong and suggest fixes
4. **Minimal API**: Small public surface area that's easy to understand
5. **No assumptions**: Works with any project structure defined in config
6. **Production-ready**: Designed for external users with comprehensive documentation

## Example: Building an xtask

Here's how you might use this library in an `xtask` deployment command:

```rust
use rust_iac_xtask_core::{IaCConfig, ConfigError};
use std::path::Path;

fn deploy_command(env_name: &str) -> Result<(), ConfigError> {
    // Load configuration
    let config = IaCConfig::from_file(Path::new("iac-config.yaml"))?;

    // Find the environment
    let env = config.find_environment(env_name).ok_or_else(|| {
        ConfigError::EnvironmentNotFound(
            env_name.to_string(),
            config.environment_names().join(", "),
        )
    })?;

    println!("Deploying to {} environment", env.name);
    println!("Manifests path: {}", env.manifests_path.display());

    if env.requires_kustomize {
        println!("Using Kustomize for deployment");
        // Run: kubectl apply -k {}
    } else {
        println!("Using kubectl apply");
        // Run: kubectl apply -f {}
    }

    Ok(())
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with verbose output:

```bash
cargo test -- --nocapture
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! This library is designed to be reusable across projects, so please ensure:

1. No hardcoded assumptions about specific projects
2. Clear, actionable error messages
3. Comprehensive documentation
4. Tests for new functionality
5. No panics in library code
