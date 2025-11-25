#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

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
    }
}
