use axum::{Json, Router, extract::State, routing::get};
use business_core::governance::TaskService;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHint {
    pub id: String,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHintsResponse {
    pub next_tasks: Vec<AgentHint>,
}

pub fn router(state: AppState) -> Router {
    Router::new().route("/platform/agent/hints", get(agent_hints)).with_state(state)
}

async fn agent_hints(
    State(state): State<AppState>,
) -> Result<Json<AgentHintsResponse>, crate::AppError> {
    let service = TaskService::new(state.governance_repo.clone());
    let tasks = service.list_tasks().map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to list tasks: {}", e),
        )
    })?;

    // Simple heuristic: prioritize Todo and InProgress tasks
    let hints: Vec<AgentHint> = tasks
        .into_iter()
        .filter(|t| {
            matches!(
                t.status,
                business_core::governance::TaskStatus::Todo
                    | business_core::governance::TaskStatus::InProgress
            )
        })
        .map(|t| AgentHint {
            id: t.id.0.clone(),
            status: format!("{:?}", t.status),
            reason: format!("Task '{}' is ready for work", t.title),
        })
        .collect();

    Ok(Json(AgentHintsResponse { next_tasks: hints }))
}
