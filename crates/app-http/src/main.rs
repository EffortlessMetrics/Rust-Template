use app_http::{AppState, app_with_state, resolve_workspace_root};
use std::net::SocketAddr;
use tracing::{error, info};

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

    // Run database migrations if enabled
    if should_run_migrations(&validated_config) {
        info!("Database auto-migration enabled, running migrations...");
        if let Err(err) = run_database_migrations().await {
            error!("Database migration failed: {}", err);
            eprintln!("Database migration failed: {}", err);
            std::process::exit(1);
        }
        info!("Database migrations completed successfully");
    } else {
        info!("Database auto-migration disabled, skipping migrations");
    }

    // Initialize governance repository
    let specs_dir = workspace_root.join("specs");
    let governance_repo =
        std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

    // Build our application router from lib, reusing validated config
    let app_state = match AppState::with_config(
        governance_repo,
        workspace_root.clone(),
        Some(validated_config.clone()),
    ) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Platform auth configuration error: {}", err);
            std::process::exit(1);
        }
    };
    let app = app_with_state(app_state);

    // Start server on the documented platform port
    let addr = SocketAddr::from(([0, 0, 0, 0], validated_config.http_port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(app_http::shutdown::shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Check if database migrations should be run based on configuration
fn should_run_migrations(config: &spec_runtime::ValidatedConfig) -> bool {
    config.settings.get("database.auto_migrate").and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Run database migrations using the existing database configuration
async fn run_database_migrations() -> anyhow::Result<()> {
    // Create a database pool using the existing configuration
    let pool = adapters_db_sqlx::create_pool().await?;

    // Run migrations
    adapters_db_sqlx::run_migrations(&pool).await?;

    Ok(())
}
