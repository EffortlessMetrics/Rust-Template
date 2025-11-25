use app_http::{AppState, app_with_state, resolve_workspace_root};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry (tracing)
    telemetry::init_tracing("app-http");

    info!("Starting HTTP service");

    let workspace_root = resolve_workspace_root();

    let config_path = workspace_root.join("config/local.yaml");
    let schema_path = workspace_root.join("specs/config_schema.yaml");

    let validated_config = match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!(
                "Configuration error (config: {}, schema: {}): {}",
                config_path.display(),
                schema_path.display(),
                err
            );
            std::process::exit(1);
        }
    };

    // Initialize governance repository
    let specs_dir = workspace_root.join("specs");
    let governance_repo =
        std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

    // Build our application router from lib, reusing validated config
    let app_state = AppState::with_config(
        governance_repo,
        workspace_root.clone(),
        Some(validated_config.clone()),
    );
    let app = app_with_state(app_state);

    // Start server on the documented platform port
    let addr = SocketAddr::from(([0, 0, 0, 0], validated_config.http_port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
