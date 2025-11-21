use axum::{Json, Router, extract::State, routing::get};
use business_core::governance::{GovernanceRepository, TaskService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub fn router(repo: Arc<dyn GovernanceRepository>) -> Router {
    Router::new().route("/platform/agent/hints", get(agent_hints)).with_state(repo)
}

async fn agent_hints(
    State(repo): State<Arc<dyn GovernanceRepository>>,
) -> Result<Json<AgentHintsResponse>, crate::AppError> {
    let service = TaskService::new(repo);
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
