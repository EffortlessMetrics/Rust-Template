use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use business_core::governance::TaskService;
use serde::{Deserialize, Serialize};
use spec_runtime::hints::{self, HintEngine, HintLinks, HintReason, HintTarget};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendedStep {
    pub kind: String,
    pub value: String,
}

/// Wire format for agent hints (AC-TPL-AGENT-HINTS-SCHEMA).
///
/// Uses the canonical `Hint*` types from `spec_runtime::hints` for schema fields,
/// plus convenience fields for backward compatibility with AC-TPL-AGENT-HINTS.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHint {
    // Full schema fields (AC-TPL-AGENT-HINTS-SCHEMA) - canonical types from spec_runtime
    pub id: String,
    pub kind: String,
    pub priority: String,
    pub status: String,
    pub reason: HintReason,
    pub target: HintTarget,
    pub tags: Vec<String>,
    pub links: HintLinks,

    // Convenience fields (backward compatibility with AC-TPL-AGENT-HINTS)
    pub task_id: String,
    pub title: String,
    pub owner: String,
    pub labels: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
    pub recommended_sequence: Vec<RecommendedStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHintsResponse {
    pub hints: Vec<AgentHint>,
    /// Warnings about referential integrity issues (invalid AC/REQ references)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<spec_runtime::ReferentialWarning>,
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

    // Load spec_ledger for referential integrity validation (AC-TPL-HINTS-REFERENTIAL-INTEGRITY)
    let ledger_path = state.workspace_root.join("specs/spec_ledger.yaml");
    let ledger = spec_runtime::load_spec_ledger(&ledger_path).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to load spec_ledger: {}", e),
        )
    })?;
    let valid_ac_ids = spec_runtime::build_ac_id_index(&ledger);
    let valid_req_ids = spec_runtime::build_req_id_index(&ledger);

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

    // Create HintEngine with AC coverage and referential integrity validation
    let mut engine =
        HintEngine::with_validation(ac_index.clone(), runtime_tasks, valid_ac_ids, valid_req_ids);

    // Build kernel AC statuses for governance hints (AC-TPL-HINTS-KERNEL-SIGNALS)
    let kernel_acs = spec_runtime::build_kernel_ac_statuses(&ledger, &ac_index);
    engine.set_kernel_acs(kernel_acs);

    let hint_engine_hints = engine.task_hints();
    let kernel_governance_hints = engine.kernel_governance_hints();

    // Collect any referential integrity warnings
    let warnings: Vec<spec_runtime::ReferentialWarning> = engine.warnings().to_vec();

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

            // Convert status
            let status_str = match hint.status {
                hints::HintStatus::Open => "open".to_string(),
                hints::HintStatus::InProgress => "in_progress".to_string(),
                hints::HintStatus::Done => "done".to_string(),
            };

            // Convert priority
            let priority_str = match hint.priority {
                hints::HintPriority::Low => "low".to_string(),
                hints::HintPriority::Medium => "medium".to_string(),
                hints::HintPriority::High => "high".to_string(),
            };

            // Convert kind
            let kind_str = match hint.kind {
                hints::HintKind::Task => "task".to_string(),
                hints::HintKind::Governance => "governance".to_string(),
                hints::HintKind::Policy => "policy".to_string(),
                hints::HintKind::Flow => "flow".to_string(),
            };

            // Reuse links and target directly from core hint (no conversion needed)
            let links = hint.links.clone();
            let target = hint.target.clone();

            Some(AgentHint {
                // Full schema fields - reuse core types directly
                id: hint.id.clone(),
                kind: kind_str,
                priority: priority_str,
                status: status_str,
                reason: hint.reason.clone(),
                target,
                tags: hint.tags.clone(),
                links,

                // Backward-compatible convenience fields
                task_id,
                title: hint.title.clone(),
                owner: definition.owner.clone().unwrap_or_else(|| "unassigned".to_string()),
                labels: hint.tags.clone(), // Mirror tags for legacy BDD/UI compatibility
                requirement_ids: vec![definition.requirement.clone()],
                ac_ids: definition.acs.clone(),
                recommended_sequence,
            })
        })
        .collect();

    // Add kernel governance hints for failing kernel ACs (AC-TPL-HINTS-KERNEL-SIGNALS)
    // These are high-priority hints to fix failing kernel ACs before other work
    for hint in &kernel_governance_hints {
        let ac_id = match &hint.target {
            hints::HintTarget::Ac { id } => id.clone(),
            _ => continue,
        };

        hints.push(AgentHint {
            // Full schema fields
            id: hint.id.clone(),
            kind: "governance".to_string(),
            priority: "high".to_string(),
            status: "open".to_string(),
            reason: hint.reason.clone(),
            target: hint.target.clone(),
            tags: hint.tags.clone(),
            links: hint.links.clone(),

            // Convenience fields for governance hints (AC-focused)
            task_id: ac_id.clone(), // Use AC ID as task_id for compatibility
            title: hint.title.clone(),
            owner: "kernel".to_string(), // Kernel ACs are owned by the kernel
            labels: hint.tags.clone(),
            requirement_ids: vec![hint.reason.details.clone()], // Contains REQ info
            ac_ids: vec![ac_id],
            recommended_sequence: vec![], // No flow sequence for governance hints
        });
    }

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

    Ok(Json(AgentHintsResponse { hints, warnings }))
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
