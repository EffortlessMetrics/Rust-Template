use crate::AppState;
use axum::{Json, extract::State};
use serde::Serialize;
use spec_runtime::load_service_metadata;

/// Debug info response DTO
#[derive(Debug, Serialize)]
pub(super) struct DebugInfo {
    kernel_version: String,
    template_version: String,
}

/// Platform debug info endpoint
///
/// Returns basic kernel and template version information.
/// Documented in docs/how-to/add-http-endpoint.md as a canonical example.
pub(super) async fn debug_info(State(state): State<AppState>) -> Json<DebugInfo> {
    let root = &state.workspace_root;

    let template_version = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .ok()
        .and_then(|m| m.template_version)
        .unwrap_or_else(|| "unknown".to_string());

    Json(DebugInfo { kernel_version: env!("CARGO_PKG_VERSION").to_string(), template_version })
}
