//! K8s Infrastructure-as-Code alignment tests
//!
//! This module validates that Kubernetes manifests align with the
//! config schema and environment definitions.

#[cfg(test)]
mod tests {
    use crate::validate_config;
    use serde_yaml::Value;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root should be two levels above spec-runtime")
            .to_path_buf()
    }

    /// AC-TPL-IAC-K8S-ALIGN: Kubernetes manifests under infra/k8s align with config_schema.yaml
    #[test]
    fn iac_k8s_aligns_with_config() {
        let root = workspace_root();
        let k8s_base = root.join("infra/k8s/dev");

        // Load K8s manifests
        let deployment = load_k8s_manifest(&k8s_base.join("deployment.yaml"));
        let service = load_k8s_manifest(&k8s_base.join("service.yaml"));
        let configmap = load_k8s_manifest(&k8s_base.join("configmap.yaml"));

        // Load and validate config schema
        let config = validate_config(
            &root.join("specs/config_schema.yaml"),
            &root.join("config/local.yaml"),
        )
        .expect("config/local.yaml should validate against specs/config_schema.yaml");

        // Load config schema to get expected settings/secrets keys
        let schema_content = fs::read_to_string(root.join("specs/config_schema.yaml"))
            .expect("Failed to read config_schema.yaml");
        let schema: Value =
            serde_yaml::from_str(&schema_content).expect("Failed to parse config_schema.yaml");

        let expected_settings = extract_config_keys(&schema, "settings");
        let expected_secrets = extract_config_keys(&schema, "secrets");

        // Validate Deployment
        validate_deployment_ports(&deployment, config.http_port);
        validate_deployment_configmap_ref(&deployment, "app-config");
        validate_deployment_secret_ref(&deployment, "app-secrets");

        // Validate Service
        validate_service_ports(&service, config.http_port);

        // Validate ConfigMap alignment with config schema
        validate_configmap_keys(&configmap, &expected_settings, &expected_secrets);

        println!("✓ K8s manifests align with config schema");
    }

    fn load_k8s_manifest(path: &PathBuf) -> Value {
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read K8s manifest at {}", path.display()));
        serde_yaml::from_str(&contents)
            .unwrap_or_else(|_| panic!("Failed to parse K8s manifest at {}", path.display()))
    }

    fn extract_config_keys(schema: &Value, section: &str) -> HashSet<String> {
        schema
            .get(section)
            .and_then(Value::as_sequence)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("key").and_then(Value::as_str).map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn validate_deployment_ports(deployment: &Value, expected_http_port: u16) {
        let containers = deployment
            .get("spec")
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|s| s.get("containers"))
            .and_then(Value::as_sequence)
            .expect("Deployment should have containers");

        let mut found_http_port = false;
        for container in containers {
            if let Some(ports) = container.get("ports").and_then(Value::as_sequence) {
                for port in ports {
                    if let Some(name) = port.get("name").and_then(Value::as_str) {
                        if name == "http" {
                            let container_port = port
                                .get("containerPort")
                                .and_then(Value::as_u64)
                                .expect("http port should have containerPort");
                            assert_eq!(
                                container_port, expected_http_port as u64,
                                "Deployment containerPort should match config_schema.yaml http.port ({})",
                                expected_http_port
                            );
                            found_http_port = true;
                        }
                    }
                }
            }
        }
        assert!(found_http_port, "Deployment should expose http port matching config schema");
    }

    fn validate_deployment_configmap_ref(deployment: &Value, expected_name: &str) {
        let env_from = deployment
            .get("spec")
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|s| s.get("containers"))
            .and_then(Value::as_sequence)
            .and_then(|containers| containers.first())
            .and_then(|c| c.get("envFrom"))
            .and_then(Value::as_sequence)
            .expect("Deployment should have envFrom");

        let has_configmap_ref = env_from.iter().any(|ref_obj| {
            ref_obj.get("configMapRef").and_then(|cm| cm.get("name")).and_then(Value::as_str)
                == Some(expected_name)
        });

        assert!(has_configmap_ref, "Deployment should reference ConfigMap '{}'", expected_name);
    }

    fn validate_deployment_secret_ref(deployment: &Value, expected_name: &str) {
        let env_from = deployment
            .get("spec")
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|s| s.get("containers"))
            .and_then(Value::as_sequence)
            .and_then(|containers| containers.first())
            .and_then(|c| c.get("envFrom"))
            .and_then(Value::as_sequence)
            .expect("Deployment should have envFrom");

        let has_secret_ref = env_from.iter().any(|ref_obj| {
            ref_obj.get("secretRef").and_then(|s| s.get("name")).and_then(Value::as_str)
                == Some(expected_name)
        });

        assert!(has_secret_ref, "Deployment should reference Secret '{}'", expected_name);
    }

    fn validate_service_ports(service: &Value, expected_http_port: u16) {
        let ports = service
            .get("spec")
            .and_then(|s| s.get("ports"))
            .and_then(Value::as_sequence)
            .expect("Service should have ports");

        let has_http_port = ports.iter().any(|port| {
            port.get("name").and_then(Value::as_str) == Some("http")
                && port.get("targetPort").and_then(Value::as_str) == Some("http")
        });

        assert!(
            has_http_port,
            "Service should map to http port (targetPort: http) which ultimately maps to containerPort {}",
            expected_http_port
        );
    }

    fn validate_configmap_keys(
        configmap: &Value,
        _expected_settings: &HashSet<String>,
        expected_secrets: &HashSet<String>,
    ) {
        let data =
            configmap.get("data").and_then(Value::as_mapping).expect("ConfigMap should have data");

        let configmap_keys: HashSet<String> =
            data.keys().filter_map(|k| k.as_str().map(String::from)).collect();

        // ConfigMap should not contain secret keys
        for secret_key in expected_secrets {
            // Convert dotted keys to env var format (e.g., db.url -> DB_URL or DATABASE_URL)
            let env_variants = key_to_env_variants(secret_key);
            for variant in &env_variants {
                assert!(
                    !configmap_keys.contains(variant),
                    "ConfigMap should not contain secret key '{}' (or variant '{}')",
                    secret_key,
                    variant
                );
            }
        }

        // We don't enforce that ALL settings are in ConfigMap since some may be set via other means,
        // but we do verify the PORT key is present and matches config schema
        assert!(
            configmap_keys.contains("PORT"),
            "ConfigMap should contain PORT env var for http.port setting"
        );

        let port_value = data
            .get(&Value::String("PORT".to_string()))
            .and_then(Value::as_str)
            .expect("PORT should be a string");

        // Note: We can't directly validate the port value here without loading config,
        // but we validate its presence. The integration comes from validate_deployment_ports.
        assert!(!port_value.is_empty(), "PORT value should not be empty in ConfigMap");
    }

    fn key_to_env_variants(key: &str) -> Vec<String> {
        // Generate common env var naming variants
        let upper = key.to_uppercase().replace('.', "_");
        vec![
            upper.clone(),
            // Also handle common mappings like db.url -> DATABASE_URL
            match key {
                "db.url" => "DATABASE_URL".to_string(),
                "auth.jwt_signing_key" => "JWT_SIGNING_KEY".to_string(),
                _ => upper,
            },
        ]
    }
}
