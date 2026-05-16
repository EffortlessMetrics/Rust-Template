use std::path::Path;

pub(crate) fn load_valid_config(workspace_root: &Path) -> Option<spec_runtime::ValidatedConfig> {
    let config_path = workspace_root.join("config/local.yaml");
    let schema_path = workspace_root.join("specs/config_schema.yaml");

    match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(cfg) => Some(cfg),
        Err(err) => {
            tracing::warn!(
                error = %err,
                config_path = %config_path.display(),
                "local config unavailable or invalid; continuing with environment defaults"
            );
            None
        }
    }
}
