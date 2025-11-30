use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use business_core::governance::TaskService;
use serde::{Deserialize, Serialize};
use spec_runtime::hints::{self, HintEngine};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendedStep {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHint {
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub owner: String,
    pub labels: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
    pub reason: String,
    pub recommended_sequence: Vec<RecommendedStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHintsResponse {
    pub hints: Vec<AgentHint>,
}

#[derive(Debug, Deserialize)]
pub struct HintsFilters {
    pub owner: Option<String>,
    pub label: Option<String>,
    pub requirement: Option<String>,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new().route("/platform/agent/hints", get(agent_hints)).with_state(state)
}

async fn agent_hints(
    State(state): State<AppState>,
    Query(filters): Query<HintsFilters>,
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

    // Load AC coverage from feature_status.md
    let feature_status_path = state.workspace_root.join("docs/feature_status.md");
    let ac_index = hints::parse_feature_status(&feature_status_path).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to parse feature_status.md: {}", e),
        )
    })?;

    // Load devex_flows.yaml for flow-based command sequences
    let devex_path = state.workspace_root.join("specs/devex_flows.yaml");
    let devex_content = std::fs::read_to_string(&devex_path).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to read devex_flows.yaml: {}", e),
        )
    })?;
    let devex_spec: serde_yaml::Value = serde_yaml::from_str(&devex_content).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to parse devex_flows.yaml: {}", e),
        )
    })?;

    // Convert governance tasks to spec_runtime tasks
    let runtime_tasks: Vec<spec_runtime::Task> = tasks
        .iter()
        .filter_map(|t| {
            let task_id = t.id.0.clone();
            let definition = task_definitions.get(&task_id)?;

            Some(spec_runtime::Task {
                id: task_id,
                title: definition.title.clone(),
                status: format!("{:?}", t.status),
                requirement: definition.requirement.clone(),
                acs: definition.acs.clone(),
                labels: definition.labels.clone(),
                owner: definition.owner.clone(),
                docs: None,
                summary: definition.summary.clone().unwrap_or_default(),
                recommended_flows: definition.recommended_flows.clone(),
                depends_on: vec![],
            })
        })
        .collect();

    // Create HintEngine with AC coverage
    let engine = HintEngine::new(ac_index, runtime_tasks);
    let hint_engine_hints = engine.task_hints();

    // Convert HintEngine hints to AgentHints and build recommended sequences
    let mut hints: Vec<AgentHint> = hint_engine_hints
        .iter()
        .filter_map(|hint| {
            // Only include Todo and InProgress hints (HintEngine filters these)
            if !matches!(hint.status, hints::HintStatus::Open | hints::HintStatus::InProgress) {
                return None;
            }

            let task_id = match &hint.target {
                hints::HintTarget::Task { id } => id.clone(),
                _ => return None,
            };

            let definition = task_definitions.get(&task_id)?;

            // Build recommended sequence from recommended_flows
            let recommended_sequence = build_recommended_sequence(
                &task_id,
                &definition.recommended_flows,
                &definition.acs,
                &devex_spec,
            );

            Some(AgentHint {
                task_id,
                title: hint.title.clone(),
                status: match hint.status {
                    hints::HintStatus::Open => "open".to_string(),
                    hints::HintStatus::InProgress => "in_progress".to_string(),
                    hints::HintStatus::Done => "done".to_string(),
                },
                owner: definition.owner.clone().unwrap_or_else(|| "unassigned".to_string()),
                labels: hint.tags.clone(),
                requirement_ids: vec![definition.requirement.clone()],
                ac_ids: definition.acs.clone(),
                reason: hint.reason.details.clone(),
                recommended_sequence,
            })
        })
        .collect();

    // Apply filters
    hints.retain(|hint| {
        // Filter by owner
        if let Some(ref owner_filter) = filters.owner
            && !hint.owner.eq_ignore_ascii_case(owner_filter)
        {
            return false;
        }

        // Filter by label
        if let Some(ref label_filter) = filters.label
            && !hint.labels.iter().any(|l| l.eq_ignore_ascii_case(label_filter))
        {
            return false;
        }

        // Filter by requirement
        if let Some(ref req_filter) = filters.requirement
            && !hint.requirement_ids.iter().any(|r| r.eq_ignore_ascii_case(req_filter))
        {
            return false;
        }

        true
    });

    // Sort by: 1) status (in_progress before open), 2) priority label, 3) ID
    hints.sort_by(|a, b| {
        // Primary: status (in_progress before open)
        let status_order_a = if a.status == "in_progress" { 0 } else { 1 };
        let status_order_b = if b.status == "in_progress" { 0 } else { 1 };

        match status_order_a.cmp(&status_order_b) {
            std::cmp::Ordering::Equal => {
                // Secondary: priority label (high > medium > low > none)
                let priority_a = get_priority_order(&a.labels);
                let priority_b = get_priority_order(&b.labels);

                match priority_a.cmp(&priority_b) {
                    std::cmp::Ordering::Equal => {
                        // Tertiary: ID (alphabetical)
                        a.task_id.cmp(&b.task_id)
                    }
                    other => other,
                }
            }
            other => other,
        }
    });

    Ok(Json(AgentHintsResponse { hints }))
}

/// Helper function to determine priority order from labels
/// Returns 0 for highest priority (priority:high), higher numbers for lower priority
fn get_priority_order(labels: &[String]) -> u8 {
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        if label_lower == "priority:high" || label_lower == "high" {
            return 0;
        } else if label_lower == "priority:medium" || label_lower == "medium" {
            return 1;
        } else if label_lower == "priority:low" || label_lower == "low" {
            return 2;
        }
    }
    // No priority label = lowest priority
    3
}

/// Build recommended command sequence from task's recommended_flows
fn build_recommended_sequence(
    task_id: &str,
    recommended_flows: &[String],
    ac_ids: &[String],
    devex_spec: &serde_yaml::Value,
) -> Vec<RecommendedStep> {
    let mut sequence = Vec::new();

    // Extract flows map from devex_spec
    let flows = match devex_spec.get("flows") {
        Some(serde_yaml::Value::Mapping(m)) => m,
        _ => return sequence,
    };

    // Process each recommended flow
    for flow_name in recommended_flows {
        if let Some(flow_value) = flows.get(flow_name)
            && let Some(steps_value) = flow_value.get("steps")
            && let Some(steps_seq) = steps_value.as_sequence()
        {
            // Add each step as a command
            for step in steps_seq {
                if let Some(cmd) = step.as_str() {
                    let command_value = match cmd {
                        // Special handling for common commands with task-specific params
                        "bundle" => format!("cargo xtask bundle {}", task_id),
                        "test-ac" => {
                            if let Some(first_ac) = ac_ids.first() {
                                format!("cargo xtask test-ac {}", first_ac)
                            } else {
                                format!("cargo xtask {}", cmd)
                            }
                        }
                        "bdd" => "cargo xtask bdd".to_string(),
                        "selftest" => "cargo xtask selftest".to_string(),
                        "ac-new" => "cargo xtask ac-new".to_string(),
                        "adr-new" => "cargo xtask adr-new".to_string(),
                        "adr-check" => "cargo xtask adr-check".to_string(),
                        "audit" => "cargo xtask audit".to_string(),
                        "release-prepare" => "cargo xtask release-prepare".to_string(),
                        "release-verify" => "cargo xtask release-verify".to_string(),
                        _ => format!("cargo xtask {}", cmd),
                    };

                    sequence.push(RecommendedStep {
                        kind: "command".to_string(),
                        value: command_value,
                    });
                }
            }
        }
    }

    sequence
}
