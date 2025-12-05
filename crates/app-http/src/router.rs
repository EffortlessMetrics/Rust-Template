use crate::{
    agent, health, metrics, middleware, platform, tasks, version, echo, AppState,
};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub fn build_router(app_state: AppState) -> Router {
    let auth_state = app_state.clone();
    let platform_state = app_state.clone();
    let platform_router = Router::new()
        .with_state(platform_state.clone())
        .merge(platform::router(platform_state.clone()))
        .route("/tasks/{id}/status", post(tasks::update_task_status))
        .layer(axum::middleware::from_fn_with_state(auth_state, middleware::platform_auth_guard))
        .with_state(platform_state.clone());

    let tasks_router =
        Router::new().with_state(app_state.clone()).route("/ui/tasks", get(tasks::tasks_ui));

    let agent_router = agent::router(app_state.clone());

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform_router)
        // Platform UI routes (at root level)
        .merge(platform::ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        // Middleware layers (applied in reverse order - bottom to top)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        .layer(
            // Configure TraceLayer to include request_id field
            TraceLayer::new_for_http().make_span_with(|request: &axum::extract::Request| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = tracing::field::Empty, // Will be filled by request_id middleware
                )
            }),
        )
        .with_state(app_state)
}
