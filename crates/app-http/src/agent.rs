use axum::{Json, Router, extract::State, routing::get};
use business_core::governance::TaskService;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendedStep {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHint {
    pub task_id: String,
    pub status: String,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
    pub reason: String,
    pub recommended_sequence: Vec<RecommendedStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHintsResponse {
    pub hints: Vec<AgentHint>,
}

pub fn router(state: AppState) -> Router<AppState> {
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

    // Load full task definitions from tasks.yaml for rich metadata
    let tasks_path = state.workspace_root.join("specs/tasks.yaml");
    let task_definitions = adapters_spec_fs::tasks_def::load_tasks_definitions(&tasks_path)
        .map_err(|e| {
            crate::AppError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                crate::ErrorCode::InternalError,
                format!("Failed to load task definitions: {}", e),
            )
        })?;

    // Filter for Todo and InProgress tasks, and enrich with metadata
    let hints: Vec<AgentHint> = tasks
        .into_iter()
        .filter(|t| {
            matches!(
                t.status,
                business_core::governance::TaskStatus::Todo
                    | business_core::governance::TaskStatus::InProgress
            )
        })
        .filter_map(|t| {
            let task_id = t.id.0.clone();
            let definition = task_definitions.get(&task_id)?;

            // Build recommended sequence from recommended_flows
            let recommended_sequence = if !definition.recommended_flows.is_empty() {
                vec![
                    RecommendedStep {
                        kind: "command".to_string(),
                        value: format!("cargo xtask bundle {}", task_id),
                    },
                    RecommendedStep {
                        kind: "command".to_string(),
                        value: format!(
                            "cargo xtask test-ac {}",
                            definition.acs.first().unwrap_or(&"<AC-ID>".to_string())
                        ),
                    },
                ]
            } else {
                vec![]
            };

            Some(AgentHint {
                task_id: task_id.clone(),
                status: format!("{:?}", t.status),
                requirement_ids: vec![definition.requirement.clone()],
                ac_ids: definition.acs.clone(),
                reason: format!("Task '{}' is ready for work", t.title),
                recommended_sequence,
            })
        })
        .collect();

    Ok(Json(AgentHintsResponse { hints }))
}
