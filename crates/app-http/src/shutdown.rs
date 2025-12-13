//! Graceful shutdown signal handling.
//!
//! Provides cross-platform signal handling for graceful HTTP server shutdown.
//! Handles Ctrl-C on all platforms and SIGTERM on Unix systems.
//!
//! # Example
//!
//! ```ignore
//! axum::serve(listener, app)
//!     .with_graceful_shutdown(shutdown_signal())
//!     .await?;
//! ```

use tokio::signal;
use tracing::info;

/// Create a future that completes when a shutdown signal is received.
///
/// On Unix: Handles both SIGTERM and SIGINT (Ctrl-C)
/// On Windows: Handles Ctrl-C only
///
/// When the signal is received, logs an informational message and returns,
/// allowing the server to begin graceful shutdown.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl-C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}
