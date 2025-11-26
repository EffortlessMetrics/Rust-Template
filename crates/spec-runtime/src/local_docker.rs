#[cfg(test)]
mod tests {
    use crate::validate_config;
    use serde_yaml::{Mapping, Value};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use url::Url;

    fn workspace_root() -> PathBuf {
        // spec-runtime lives at crates/spec-runtime; walk up twice to hit the workspace root
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root should be two levels above spec-runtime")
            .to_path_buf()
    }

    #[test]
    fn local_docker_compose_exists_and_has_core_services() {
        let root = workspace_root();
        let compose_path = root.join("docker-compose.yaml");

        assert!(
            compose_path.is_file(),
            "Expected docker-compose.yaml at {} for AC-TPL-LOCAL-DOCKER",
            compose_path.display()
        );

        let contents =
            fs::read_to_string(&compose_path).expect("Failed to read docker-compose.yaml");

        assert!(
            contents.contains("postgres"),
            "docker-compose.yaml should define a postgres service"
        );
        assert!(contents.contains("jaeger"), "docker-compose.yaml should define a jaeger service");
        assert!(
            contents.contains("5432"),
            "docker-compose.yaml should expose postgres on the expected port"
        );
        assert!(
            contents.contains("4317") || contents.contains("16686") || contents.contains("4318"),
            "docker-compose.yaml should expose jaeger on an expected port"
        );

        assert!(
            contents.contains("app:"),
            "docker-compose.yaml should define an app service to bind the HTTP port"
        );
    }

    #[test]
    fn iac_compose_aligns_with_config() {
        let root = workspace_root();
        let compose = load_compose(&root.join("docker-compose.yaml"));
        let services = compose
            .get("services")
            .and_then(Value::as_mapping)
            .expect("services mapping should exist");

        // Validate config to derive expected env/port defaults
        let config = validate_config(
            &root.join("specs/config_schema.yaml"),
            &root.join("config/local.yaml"),
        )
        .expect("config/local.yaml should validate against specs/config_schema.yaml");

        let postgres = get_service(services, "postgres");
        let jaeger = get_service(services, "jaeger");
        let app = get_service(services, "app");

        // Postgres env matches db.url
        let db_url = config.secrets.get("db.url").expect("db.url secret should exist");
        let db_uri = Url::parse(db_url).expect("db.url should be a valid URL");
        let expected_user = db_uri.username();
        let expected_password = db_uri.password().unwrap_or_default();
        let expected_db = db_uri.path().trim_start_matches('/');
        let expected_port = db_uri.port().unwrap_or(5432);

        let postgres_env = env_map(postgres);
        assert_env_contains(&postgres_env, "POSTGRES_USER", expected_user);
        assert_env_contains(&postgres_env, "POSTGRES_PASSWORD", expected_password);
        assert_env_contains(&postgres_env, "POSTGRES_DB", expected_db);

        let postgres_ports = port_list(postgres);
        assert!(
            postgres_ports
                .iter()
                .any(|p| p.starts_with(&format!("{expected_port}:"))
                    || p == &expected_port.to_string()),
            "Postgres service should expose host port {expected_port}"
        );

        // Jaeger exposes the OTLP port from telemetry.otlp_endpoint
        let otlp_endpoint = config
            .settings
            .get("telemetry.otlp_endpoint")
            .and_then(Value::as_str)
            .unwrap_or("http://localhost:4317");
        let otlp_uri =
            Url::parse(otlp_endpoint).expect("telemetry.otlp_endpoint should be a valid URL");
        let otlp_port = otlp_uri.port().unwrap_or(4317);
        let jaeger_ports = port_list(jaeger);
        assert!(
            jaeger_ports.iter().any(|p| p.starts_with(&format!("{otlp_port}:"))),
            "Jaeger service should publish OTLP port {otlp_port}"
        );

        // App service binds HTTP port and uses the same env defaults as config/local.yaml
        let app_ports = port_list(app);
        assert!(
            app_ports.iter().any(|p| p.contains(&config.http_port.to_string())),
            "App service should map HTTP port {} for local dev",
            config.http_port
        );

        let app_env = env_map(app);
        assert_env_contains(&app_env, "HTTP_PORT", &config.http_port.to_string());
        assert_env_contains(&app_env, "DATABASE_URL", db_url);
        assert_env_contains(&app_env, "OTEL_EXPORTER_OTLP_ENDPOINT", otlp_endpoint);
    }

    fn load_compose(path: &PathBuf) -> Value {
        let contents = fs::read_to_string(path).expect("Failed to read docker-compose.yaml");
        serde_yaml::from_str(&contents).expect("Failed to parse docker-compose.yaml")
    }

    fn get_service<'a>(services: &'a Mapping, name: &str) -> &'a Value {
        services
            .get(Value::from(name))
            .unwrap_or_else(|| panic!("Service '{}' must exist in docker-compose.yaml", name))
    }

    fn env_map(service: &Value) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(env) = service.get("environment") {
            match env {
                Value::Mapping(map) => {
                    for (k, v) in map {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            result.insert(key.to_string(), val.to_string());
                        }
                    }
                }
                Value::Sequence(seq) => {
                    for item in seq {
                        if let Some(s) = item.as_str()
                            && let Some((k, v)) = s.split_once('=')
                        {
                            result.insert(k.to_string(), v.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        result
    }

    fn port_list(service: &Value) -> Vec<String> {
        service
            .get("ports")
            .and_then(Value::as_sequence)
            .map(|ports| ports.iter().filter_map(|p| p.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    }

    fn assert_env_contains(env: &HashMap<String, String>, key: &str, expected: &str) {
        let Some(value) = env.get(key) else {
            panic!("Expected environment variable '{}' to be defined", key);
        };

        assert!(
            value.contains(expected),
            "Expected env {} to include '{}', got '{}'",
            key,
            expected,
            value
        );
    }

    #[test]
    fn iac_tf_aligns_with_config() {
        let root = workspace_root();
        let tf_path = root.join("infra/tf/main.tf");

        assert!(
            tf_path.is_file(),
            "Expected Terraform config at {} for AC-TPL-IAC-TF-ALIGN",
            tf_path.display()
        );

        let tf_contents = fs::read_to_string(&tf_path).expect("Failed to read main.tf");

        // Load config schema to get expected variables
        let schema_path = root.join("specs/config_schema.yaml");
        let schema_contents =
            fs::read_to_string(&schema_path).expect("Failed to read config_schema.yaml");
        let schema: Value =
            serde_yaml::from_str(&schema_contents).expect("Failed to parse config_schema.yaml");

        // Validate settings are present as Terraform variables
        let settings = schema
            .get("settings")
            .and_then(Value::as_sequence)
            .expect("config_schema.yaml should have settings array");

        for setting in settings {
            let key = setting.get("key").and_then(Value::as_str).expect("setting should have key");
            let tf_var_name = key.replace('.', "_");

            assert!(
                tf_contents.contains(&format!("variable \"{}\"", tf_var_name)),
                "Terraform should define variable '{}' matching config setting '{}'",
                tf_var_name,
                key
            );

            // Check default value if present
            if let Some(default) = setting.get("default") {
                let default_str = match default {
                    Value::String(s) => format!("\"{}\"", s),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                };

                assert!(
                    tf_contents.contains(&format!("default     = {}", default_str))
                        || tf_contents.contains(&format!("default = {}", default_str)),
                    "Terraform variable '{}' should have default value {}",
                    tf_var_name,
                    default_str
                );
            }
        }

        // Validate secrets are present as Terraform variables
        let secrets = schema
            .get("secrets")
            .and_then(Value::as_sequence)
            .expect("config_schema.yaml should have secrets array");

        for secret in secrets {
            let key = secret.get("key").and_then(Value::as_str).expect("secret should have key");
            let tf_var_name = key.replace('.', "_");

            assert!(
                tf_contents.contains(&format!("variable \"{}\"", tf_var_name)),
                "Terraform should define variable '{}' matching config secret '{}'",
                tf_var_name,
                key
            );

            // Secrets should be marked as sensitive
            let var_block_start = tf_contents
                .find(&format!("variable \"{}\"", tf_var_name))
                .expect("variable should exist");
            let var_block_end = tf_contents[var_block_start..]
                .find('}')
                .map(|i| var_block_start + i)
                .expect("variable block should close");
            let var_block = &tf_contents[var_block_start..var_block_end];

            assert!(
                var_block.contains("sensitive   = true") || var_block.contains("sensitive = true"),
                "Terraform variable '{}' should be marked as sensitive",
                tf_var_name
            );
        }
    }
}
