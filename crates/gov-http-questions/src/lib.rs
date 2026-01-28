//! Question endpoints for tracking design decisions and ambiguities.
//!
//! Questions capture flow decision points, options, recommendations,
//! and their resolutions during development.
//!
//! # Endpoints
//!
//! - `GET /questions` - List all questions (with optional status filter)
//! - `GET /questions/{id}` - Get a specific question

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use gov_http_core::{PlatformError, PlatformState};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export types from gov-http-types for backwards compatibility
pub use gov_http_types::{
    Question, QuestionContext, QuestionOption, QuestionResolution, Recommendation,
};

/// Response for listing questions.
#[derive(Debug, Serialize)]
pub struct QuestionsListResponse {
    /// List of question summaries.
    pub questions: Vec<QuestionSummary>,
    /// Total number of questions.
    pub total: usize,
}

/// Summary of a question for list views.
#[derive(Debug, Serialize)]
pub struct QuestionSummary {
    /// Unique identifier (e.g., "Q-FLOW-001").
    pub id: String,
    /// Brief summary of the question.
    pub summary: String,
    /// Current status (e.g., "open", "resolved").
    pub status: String,
    /// Associated flow (e.g., "bundle").
    pub flow: String,
    /// Phase within the flow (e.g., "selection").
    pub phase: String,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct QuestionFilters {
    pub status: Option<String>,
}

/// Router for question endpoints.
///
/// Returns a router that handles:
/// - `GET /questions` - List all questions (with optional status filter)
/// - `GET /questions/{id}` - Get a specific question
pub fn router<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/questions", get(get_all_questions::<S>))
        .route("/questions/{id}", get(get_question_by_id::<S>))
}

/// Load all question entries from questions/ directory.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_all_questions(
    root: &std::path::Path,
    status_filter: Option<&str>,
) -> Result<Vec<Question>, PlatformError> {
    let questions_dir = root.join("questions");

    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    let dir_entries = std::fs::read_dir(&questions_dir).map_err(|e| {
        PlatformError::internal(format!("Failed to read questions directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
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

/// Load a single question entry from a YAML file.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_question_entry(path: &std::path::Path) -> Result<Question, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read question file {}: {}", path.display(), e))
    })?;

    let question: Question = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse question YAML {}: {}", path.display(), e))
    })?;

    Ok(question)
}

/// GET /questions - Get all question entries (optionally filtered by status)
async fn get_all_questions<S>(
    State(state): State<S>,
    Query(filters): Query<QuestionFilters>,
) -> Result<Json<QuestionsListResponse>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();
    let status_filter = filters.status.clone();

    // Offload blocking filesystem I/O to spawn_blocking to avoid starving
    // the Tokio async runtime under concurrent load.
    let questions =
        tokio::task::spawn_blocking(move || load_all_questions(&root, status_filter.as_deref()))
            .await
            .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

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

/// Load a question entry by ID.
///
/// This function performs blocking filesystem I/O and should be called from
/// within `spawn_blocking` to avoid starving the Tokio async runtime.
fn load_question_by_id(
    questions_dir: &std::path::Path,
    id: &str,
) -> Result<Question, PlatformError> {
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

            return Ok(question);
        }
    }

    Err(PlatformError::not_found(format!("Question '{}' not found", id)))
}

/// GET /questions/{id} - Get a specific question entry by ID
async fn get_question_by_id<S>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<Question>, PlatformError>
where
    S: PlatformState,
{
    let questions_dir: PathBuf = state.context().root().join("questions");

    // Offload blocking filesystem I/O to spawn_blocking to avoid starving
    // the Tokio async runtime under concurrent load.
    let question = tokio::task::spawn_blocking(move || load_question_by_id(&questions_dir, &id))
        .await
        .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(Json(question))
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
