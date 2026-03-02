//! IDP snapshot endpoint for platform introspection.

use axum::{Json, Router, extract::State, routing::get};
use http_errors::HttpError;
use http_idp_snapshot::{IdpSnapshot, generate_snapshot};
use tracing::instrument;

/// Create the IDP router.
pub fn router<S>() -> Router<S>
where
    S: super::PlatformState + Clone + Send + Sync + 'static,
{
    Router::<S>::new().route("/idp/snapshot", get(get_idp_snapshot::<S>))
}

/// GET /platform/idp/snapshot - Get IDP snapshot with governance health and task hints.
#[allow(clippy::result_large_err)]
#[instrument(skip(state))]
pub async fn get_idp_snapshot<S>(State(state): State<S>) -> Result<Json<IdpSnapshot>, HttpError>
where
    S: super::PlatformState,
{
    let snapshot = generate_snapshot(state.workspace_root()).map_err(|e| {
        HttpError::internal_error(format!("Failed to generate IDP snapshot: {}", e))
    })?;
    Ok(Json(snapshot))
}
