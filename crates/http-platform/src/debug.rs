use crate::PlatformState;
use axum::{Json, extract::State};
use serde::Serialize;
use spec_runtime::load_service_metadata;
use tracing::instrument;

/// Debug info response DTO.
#[derive(Debug, Clone, Serialize)]
pub struct DebugInfo {
    /// Kernel version
    pub kernel_version: String,
    /// Template version
    pub template_version: String,
}

/// Platform debug info endpoint.
///
/// Returns basic kernel and template version information.
#[instrument(skip(state))]
pub(super) async fn debug_info<S>(State(state): State<S>) -> Json<DebugInfo>
where
    S: PlatformState,
{
    let root = state.workspace_root();

    let template_version = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .ok()
        .and_then(|m| m.template_version)
        .unwrap_or_else(|| "unknown".to_string());

    Json(DebugInfo { kernel_version: env!("CARGO_PKG_VERSION").to_string(), template_version })
}
