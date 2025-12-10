use crate::{AppError, AppState};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Question artifact representing flow decision points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub req_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ac_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    pub summary: String,
    pub context: QuestionContext,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<QuestionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<Recommendation>,
    pub created_by: String,
    pub created_at: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<QuestionResolution>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    pub flow: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default = "default_reversible")]
    pub reversible: bool,
}

fn default_reversible() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub option_label: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResolution {
    pub resolved_by: String,
    pub resolved_at: String,
    pub chosen_option: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuestionsListResponse {
    pub questions: Vec<QuestionSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct QuestionSummary {
    pub id: String,
    pub summary: String,
    pub status: String,
    pub flow: String,
    pub phase: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct QuestionFilters {
    pub status: Option<String>,
}

/// Router for question endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/questions", get(get_all_questions))
        .route("/questions/{id}", get(get_question_by_id))
}

/// Load all question entries from questions/ directory
#[allow(clippy::result_large_err)]
fn load_all_questions(
    workspace_root: &std::path::Path,
    status_filter: Option<&str>,
) -> Result<Vec<Question>, AppError> {
    let questions_dir = workspace_root.join("questions");

    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    let dir_entries = fs::read_dir(&questions_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to read questions directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            AppError::internal_error(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_question_entry(&path) {
            Ok(question) => {
                // Apply status filter if provided
                if let Some(filter_status) = status_filter {
                    if question.status.eq_ignore_ascii_case(filter_status) {
                        questions.push(question);
                    }
                } else {
                    questions.push(question);
                }
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load question entry"
                );
            }
        }
    }

    // Sort by created_at (most recent first)
    questions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(questions)
}

/// Load a single question entry from a YAML file
#[allow(clippy::result_large_err)]
fn load_question_entry(path: &std::path::Path) -> Result<Question, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::internal_error(format!("Failed to read question file {}: {}", path.display(), e))
    })?;

    let question: Question = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse question YAML {}: {}", path.display(), e))
    })?;

    Ok(question)
}

/// GET /platform/questions - Get all question entries (optionally filtered by status)
async fn get_all_questions(
    State(state): State<AppState>,
    Query(filters): Query<QuestionFilters>,
) -> Result<Json<QuestionsListResponse>, AppError> {
    let root = &state.workspace_root;
    let questions = load_all_questions(root, filters.status.as_deref())?;

    let summaries: Vec<QuestionSummary> = questions
        .iter()
        .map(|q| QuestionSummary {
            id: q.id.clone(),
            summary: q.summary.clone(),
            status: q.status.clone(),
            flow: q.context.flow.clone(),
            phase: q.context.phase.clone(),
            created_at: q.created_at.clone(),
        })
        .collect();

    let total = summaries.len();

    Ok(Json(QuestionsListResponse { questions: summaries, total }))
}

/// GET /platform/questions/:id - Get a specific question entry by ID
async fn get_question_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Question>, AppError> {
    let root = &state.workspace_root;
    let questions_dir = root.join("questions");

    // Try to find the question file
    // It could be named with or without a prefix (e.g., Q-EXAMPLE-001.yaml or QUESTION-001.yaml)
    let possible_filenames = vec![
        format!("{}.yaml", id),
        format!("Q-{}.yaml", id.trim_start_matches("Q-")),
        format!("QUESTION-{}.yaml", id.trim_start_matches("QUESTION-")),
    ];

    for filename in possible_filenames {
        let file_path = questions_dir.join(&filename);
        if file_path.exists() {
            let question = load_question_entry(&file_path)?;

            // Verify the ID matches (sanity check)
            if question.id != id {
                tracing::warn!(
                    expected_id = %id,
                    found_id = %question.id,
                    file = %file_path.display(),
                    "Question ID mismatch"
                );
            }

            return Ok(Json(question));
        }
    }

    Err(AppError::not_found(format!("Question '{}' not found", id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_serialization() {
        let question = Question {
            id: "Q-TEST-001".to_string(),
            task_id: Some("implement_feature".to_string()),
            req_ids: vec!["REQ-001".to_string()],
            ac_ids: vec!["AC-001".to_string()],
            refs: vec![],
            summary: "Test question".to_string(),
            context: QuestionContext {
                flow: "bundle".to_string(),
                phase: "selection".to_string(),
                description: Some("Test description".to_string()),
                files_involved: vec![],
            },
            options: vec![],
            recommendation: None,
            created_by: "flow".to_string(),
            created_at: "2025-11-26T00:00:00Z".to_string(),
            status: "open".to_string(),
            resolution: None,
        };

        let json = serde_json::to_string(&question).expect("question should serialize to JSON");
        assert!(json.contains("Q-TEST-001"));
        assert!(json.contains("bundle"));
    }

    #[test]
    fn test_question_deserialization() {
        let yaml = r#"
id: Q-TEST-002
task_id: implement_ac
req_ids:
  - REQ-001
ac_ids:
  - AC-001
summary: "Test question"
context:
  flow: bundle
  phase: selection
created_by: flow
created_at: "2025-11-26T00:00:00Z"
status: open
"#;

        let question: Question =
            serde_yaml::from_str(yaml).expect("YAML should deserialize to Question");
        assert_eq!(question.id, "Q-TEST-002");
        assert_eq!(question.context.flow, "bundle");
        assert_eq!(question.status, "open");
    }

    #[test]
    fn test_default_status() {
        let yaml = r#"
id: Q-TEST-003
summary: "Test"
context:
  flow: test
  phase: test
created_by: flow
created_at: "2025-11-26T00:00:00Z"
"#;

        let question: Question =
            serde_yaml::from_str(yaml).expect("YAML should deserialize to Question");
        assert_eq!(question.status, "open");
    }
}
