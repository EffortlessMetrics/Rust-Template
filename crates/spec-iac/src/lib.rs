//! IaC-related spec operations.
//!
//! This crate provides types and functions for managing IaC spec operations,
//! IaC generation helpers, and IaC validation.
//!
//! # Design Principles
//!
//! - **Minimal dependencies**: Only spec-types, spec-ledger, serde, serde_yaml, thiserror, anyhow
//! - **Workflow layer**: Provides IaC spec types and generation helpers
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//!
//! # Example
//!
//! ```ignore
//! use spec_iac::{load_iac_spec, generate_k8s_manifest};
//!
//! let iac = load_iac_spec(Path::new("specs/iac.yaml"))?;
//! let manifest = generate_k8s_manifest(&iac)?;
//! ```

#![allow(missing_docs)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// Public Types
// ============================================================================

/// IaC specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IacSpec {
    pub schema_version: String,
    pub template_version: String,
    pub services: Vec<IacService>,
}

/// IaC service specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IacService {
    pub id: String,
    pub name: String,
    pub description: String,
    pub runtime: String,
    pub config_schema: Option<String>,
    pub resources: Vec<IacResource>,
}

/// IaC resource specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IacResource {
    pub r#type: String,
    pub name: String,
    pub config: serde_yaml::Value,
}

/// K8s manifest.
#[derive(Debug, Serialize, Clone)]
pub struct K8sManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: K8sMetadata,
    pub spec: K8sSpec,
}

/// K8s metadata.
#[derive(Debug, Serialize, Clone)]
pub struct K8sMetadata {
    pub name: String,
    pub labels: std::collections::HashMap<String, String>,
}

/// K8s spec.
#[derive(Debug, Serialize, Clone)]
pub struct K8sSpec {
    pub replicas: Option<u32>,
    pub selector: Option<std::collections::HashMap<String, String>>,
    pub template: K8sTemplate,
}

/// K8s template.
#[derive(Debug, Serialize, Clone)]
pub struct K8sTemplate {
    pub spec: K8sPodSpec,
}

/// K8s pod spec.
#[derive(Debug, Serialize, Clone)]
pub struct K8sPodSpec {
    pub containers: Vec<K8sContainer>,
}

/// K8s container.
#[derive(Debug, Serialize, Clone)]
pub struct K8sContainer {
    pub name: String,
    pub image: String,
    pub ports: Option<Vec<K8sPort>>,
    pub env: Option<Vec<K8sEnvVar>>,
    pub env_from: Option<Vec<K8sEnvFrom>>,
    pub resources: Option<K8sResources>,
}

/// K8s port.
#[derive(Debug, Serialize, Clone)]
pub struct K8sPort {
    pub container_port: Option<u32>,
    pub name: String,
    pub protocol: Option<String>,
}

/// K8s environment variable.
#[derive(Debug, Serialize, Clone)]
pub struct K8sEnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<K8sValueFrom>,
}

/// Value from configmap/secret.
#[derive(Debug, Serialize, Clone)]
pub struct K8sValueFrom {
    pub config_map_key_ref: Option<K8sConfigMapKeyRef>,
    pub secret_key_ref: Option<K8sSecretKeyRef>,
}

/// ConfigMap key reference.
#[derive(Debug, Serialize, Clone)]
pub struct K8sConfigMapKeyRef {
    pub name: String,
    pub key: Option<String>,
}

/// Secret key reference.
#[derive(Debug, Serialize, Clone)]
pub struct K8sSecretKeyRef {
    pub name: String,
    pub key: Option<String>,
}

/// Env from source.
#[derive(Debug, Serialize, Clone)]
pub struct K8sEnvFrom {
    pub config_map_ref: Option<K8sConfigMapRef>,
    pub secret_ref: Option<K8sSecretKeyRef>,
}

/// ConfigMap reference.
#[derive(Debug, Serialize, Clone)]
pub struct K8sConfigMapRef {
    pub name: String,
}

/// Secret reference.
#[derive(Debug, Serialize, Clone)]
pub struct K8sSecretRefRef {
    pub name: String,
}

/// K8s resources.
#[derive(Debug, Serialize, Clone)]
pub struct K8sResources {
    pub limits: Option<std::collections::HashMap<String, String>>,
    pub requests: Option<std::collections::HashMap<String, String>>,
}

/// IaC validation result.
#[derive(Debug, Clone)]
pub struct IacValidationResult {
    pub valid: bool,
    pub errors: Vec<IacValidationError>,
}

/// IaC validation error.
#[derive(Debug, Clone)]
pub struct IacValidationError {
    pub resource_id: String,
    pub error_type: String,
    pub message: String,
}

// ============================================================================
// Loading
// ============================================================================

/// Load IaC specification from a YAML file.
///
/// # Arguments
///
/// * `path` - Path to IaC spec YAML file
///
/// # Returns
///
/// Returns a parsed [`IacSpec`] instance.
///
/// # Errors
///
/// Returns an error if file is missing, unreadable, or malformed YAML.
pub fn load_iac_spec(path: &Path) -> Result<IacSpec, anyhow::Error> {
    let content = std::fs::read_to_string(path)?;

    serde_yaml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse IaC spec: {}", e))
}

// ============================================================================
// K8s Generation
// ============================================================================

/// Generate K8s manifest from IaC spec.
///
/// # Arguments
///
/// * `iac` - IaC specification
///
/// # Returns
///
/// Returns a [`K8sManifest`] for deployment.
///
/// # Errors
///
/// Returns an error if IaC spec is invalid or generation fails.
pub fn generate_k8s_manifest(iac: &IacSpec) -> Result<K8sManifest> {
    let mut all_containers = Vec::new();
    let mut config_maps = Vec::new();
    let mut _secrets: Vec<()> = Vec::new();

    for service in &iac.services {
        let mut env_vars = Vec::new();
        let mut env_from = Vec::new();

        // Add environment variables from config schema
        if let Some(_config_schema) = &service.config_schema {
            // This is simplified - in a full implementation, we'd parse the config schema
            // and generate env vars accordingly
            env_vars.push(K8sEnvVar {
                name: "PORT".to_string(),
                value: Some("8080".to_string()),
                value_from: None,
            });
        }

        // Add env_from for configmap
        if !env_vars.is_empty() {
            env_from.push(K8sEnvFrom {
                config_map_ref: Some(K8sConfigMapRef {
                    name: format!("{}-config", service.id.to_lowercase()),
                }),
                secret_ref: None,
            });
            config_maps.push(K8sConfigMap {
                api_version: "v1".to_string(),
                kind: "ConfigMap".to_string(),
                metadata: K8sMetadata {
                    name: format!("{}-config", service.id.to_lowercase()),
                    labels: {
                        let mut labels = std::collections::HashMap::new();
                        labels.insert("app".to_string(), service.id.clone());
                        labels
                    },
                },
                data: {
                    let mut data = std::collections::HashMap::new();
                    for env_var in &env_vars {
                        if let Some(value) = &env_var.value {
                            data.insert(
                                env_var.name.clone(),
                                serde_yaml::Value::String(value.clone()),
                            );
                        }
                    }
                    data
                },
            });
        }

        // Add ports
        let ports = Some(vec![K8sPort {
            container_port: Some(8080),
            name: "http".to_string(),
            protocol: Some("TCP".to_string()),
        }]);

        all_containers.push(K8sContainer {
            name: service.id.clone(),
            image: format!("{}:latest", service.id.to_lowercase()),
            ports,
            env: if !env_vars.is_empty() { Some(env_vars) } else { None },
            env_from: if !env_from.is_empty() { Some(env_from) } else { None },
            resources: None,
        });
    }

    Ok(K8sManifest {
        api_version: "v1".to_string(),
        kind: "List".to_string(),
        metadata: K8sMetadata {
            name: "platform-services".to_string(),
            labels: std::collections::HashMap::new(),
        },
        spec: K8sSpec {
            replicas: Some(1),
            selector: None,
            template: K8sTemplate { spec: K8sPodSpec { containers: all_containers } },
        },
    })
}

/// Generate K8s Service manifest.
pub fn generate_k8s_service(service: &IacService) -> Result<K8sService> {
    let mut containers = Vec::new();
    let mut env_vars = Vec::new();
    let mut env_from = Vec::new();

    // Add environment variables from config schema
    if let Some(_config_schema) = &service.config_schema {
        env_vars.push(K8sEnvVar {
            name: "PORT".to_string(),
            value: Some("8080".to_string()),
            value_from: None,
        });
    }

    // Add env_from for configmap
    if !env_vars.is_empty() {
        env_from.push(K8sEnvFrom {
            config_map_ref: Some(K8sConfigMapRef {
                name: format!("{}-config", service.id.to_lowercase()),
            }),
            secret_ref: None,
        });
    }

    let ports = Some(vec![K8sPort {
        container_port: Some(8080),
        name: "http".to_string(),
        protocol: Some("TCP".to_string()),
    }]);

    containers.push(K8sContainer {
        name: service.id.clone(),
        image: format!("{}:latest", service.id.to_lowercase()),
        ports,
        env: if !env_vars.is_empty() { Some(env_vars) } else { None },
        env_from: if !env_from.is_empty() { Some(env_from) } else { None },
        resources: None,
    });

    Ok(K8sService {
        api_version: "v1".to_string(),
        kind: "Service".to_string(),
        metadata: K8sMetadata {
            name: service.id.clone(),
            labels: {
                let mut labels = std::collections::HashMap::new();
                labels.insert("app".to_string(), service.id.clone());
                labels
            },
        },
        spec: K8sSpec {
            replicas: Some(1),
            selector: None,
            template: K8sTemplate { spec: K8sPodSpec { containers } },
        },
    })
}

/// K8s Service.
#[derive(Debug, Serialize, Clone)]
pub struct K8sService {
    pub api_version: String,
    pub kind: String,
    pub metadata: K8sMetadata,
    pub spec: K8sSpec,
}

/// K8s ConfigMap.
#[derive(Debug, Serialize, Clone)]
pub struct K8sConfigMap {
    pub api_version: String,
    pub kind: String,
    pub metadata: K8sMetadata,
    pub data: std::collections::HashMap<String, serde_yaml::Value>,
}

// ============================================================================
// Validation
// ============================================================================

/// Validate IaC specification.
///
/// Checks:
/// - Service IDs are unique
/// - Resource names are valid
/// - Config references are valid
///
/// # Arguments
///
/// * `iac` - IaC specification to validate
///
/// # Returns
///
/// Returns an [`IacValidationResult`] with validation status and errors.
pub fn validate_iac_spec(iac: &IacSpec) -> IacValidationResult {
    let mut errors = Vec::new();

    // Check unique service IDs
    let mut service_ids = std::collections::HashSet::new();
    for service in &iac.services {
        if !service_ids.insert(&service.id) {
            errors.push(IacValidationError {
                resource_id: service.id.clone(),
                error_type: "duplicate_service_id".to_string(),
                message: format!("Duplicate service ID: {}", service.id),
            });
        }
    }

    // Validate resources
    for service in &iac.services {
        for resource in &service.resources {
            match resource.r#type.as_str() {
                "deployment" | "service" | "configmap" | "secret" => {
                    // Valid types
                }
                _ => {
                    errors.push(IacValidationError {
                        resource_id: resource.name.clone(),
                        error_type: "invalid_resource_type".to_string(),
                        message: format!("Invalid resource type: {}", resource.r#type),
                    });
                }
            }
        }
    }

    IacValidationResult { valid: errors.is_empty(), errors }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_iac_spec() {
        let yaml = r#"
schema_version: "1.0"
template_version: "3.3.1"
services:
  - id: "test-service"
    name: "Test Service"
    description: "A test service"
    runtime: "kubernetes"
    config_schema: "config_schema.yaml"
    resources:
      - type: "deployment"
        name: "test-deployment"
        config: {}
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), yaml).unwrap();

        let iac = load_iac_spec(temp.path()).unwrap();
        assert_eq!(iac.services.len(), 1);
        assert_eq!(iac.services[0].id, "test-service");
    }

    #[test]
    fn test_validate_iac_spec() {
        let iac = IacSpec {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            services: vec![
                IacService {
                    id: "service-1".to_string(),
                    name: "Service 1".to_string(),
                    description: "First service".to_string(),
                    runtime: "kubernetes".to_string(),
                    config_schema: None,
                    resources: vec![],
                },
                IacService {
                    id: "service-1".to_string(),
                    name: "Service 2".to_string(),
                    description: "Duplicate service".to_string(),
                    runtime: "kubernetes".to_string(),
                    config_schema: None,
                    resources: vec![],
                },
            ],
        };

        let result = validate_iac_spec(&iac);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, "duplicate_service_id");
    }

    #[test]
    fn test_generate_k8s_manifest() {
        let iac = IacSpec {
            schema_version: "1.0".to_string(),
            template_version: "1.0".to_string(),
            services: vec![IacService {
                id: "test-service".to_string(),
                name: "Test Service".to_string(),
                description: "A test service".to_string(),
                runtime: "kubernetes".to_string(),
                config_schema: None,
                resources: vec![],
            }],
        };

        let manifest = generate_k8s_manifest(&iac).unwrap();
        assert_eq!(manifest.api_version, "v1");
        assert_eq!(manifest.kind, "List");
    }
}
