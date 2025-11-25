use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry (tracing)
    telemetry::init_tracing("app-http");

    info!("Starting HTTP service");

    // Initialize governance repository
    let repo_root = std::env::current_dir()?;
    let specs_dir = repo_root.join("specs");
    let governance_repo =
        std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

    // Build our application router from lib
    let app = app_http::app(governance_repo);

    // Start server on the documented platform port
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
