use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry (tracing)
    telemetry::init_tracing("app-http");

    info!("Starting HTTP service");

    // Build our application router from lib
    let app = app_http::app();

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
