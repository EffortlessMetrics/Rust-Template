use crate::{
    AppState, agents_router, cors_middleware, echo, health, load_valid_config, metrics_handler,
    metrics_middleware, platform_auth_guard, platform_router, request_id_middleware,
    security_headers_middleware, tasks_ui, todos_router, ui_router, update_task_status, version,
};
use axum::{
    Router,
    routing::{get, post},
};
use business_core::governance::GovernanceRepository;
use std::path::PathBuf;
use std::sync::Arc;

/// Create the application router (reusable for both main and tests).
///
/// # Errors
///
/// Returns an error if platform auth configuration is invalid (e.g., invalid PLATFORM_AUTH_MODE).
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Result<Router, String> {
    let workspace_root = crate::resolve_workspace_root();
    let config = load_valid_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config)?;
    Ok(build_router(app_state))
}

/// Create the application router with an explicit workspace root.
/// Useful for tests to avoid reliance on global environment variables.
///
/// # Errors
///
/// Returns an error if platform auth configuration is invalid (e.g., invalid PLATFORM_AUTH_MODE).
pub fn app_with_workspace_root(
    governance_repo: Arc<dyn GovernanceRepository>,
    workspace_root: PathBuf,
) -> Result<Router, String> {
    let config = load_valid_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config)?;
    Ok(build_router(app_state))
}

/// Create an application router from an already-constructed state (e.g., when main has validated config).
pub fn app_with_state(app_state: AppState) -> Router {
    build_router(app_state)
}

fn build_router(app_state: AppState) -> Router {
    let auth_state = app_state.clone();
    let platform_state = app_state.clone();

    let platform_router = Router::new()
        .with_state(platform_state.clone())
        .merge(platform_router(platform_state.clone()))
        .route("/tasks/{id}/status", post(update_task_status::<AppState>))
        .layer(axum::middleware::from_fn_with_state(auth_state, platform_auth_guard))
        .with_state(platform_state.clone());

    let tasks_router =
        Router::new().with_state(app_state.clone()).route("/ui/tasks", get(tasks_ui::<AppState>));

    let agent_router = agents_router(app_state.clone());
    let todos_router = todos_router(app_state.clone());

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform_router)
        // Platform UI routes (at root level)
        .merge(ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        .merge(todos_router)
        // Middleware layers (applied in reverse order - bottom to top)
        // Request ID middleware (outermost - applied first to request)
        .layer(axum::middleware::from_fn(request_id_middleware))
        // Metrics middleware
        .layer(axum::middleware::from_fn(metrics_middleware))
        // CORS middleware
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), cors_middleware))
        // Security headers (innermost - applied first to response)
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), security_headers_middleware))
        .with_state(app_state)
}
