//! Question endpoints for tracking design decisions and ambiguities.
//!
//! Questions capture flow decision points, options, recommendations,
//! and their resolutions during development.
//!
//! # Endpoints
//!
//! - `GET /questions` - List all questions (with optional status filter)
//! - `POST /questions` - Create a new question
//! - `GET /questions/{id}` - Get a specific question
//! - `PUT /questions/{id}` - Resolve a question
//! - `DELETE /questions/{id}` - Delete a question

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use chrono::Utc;
use gov_http_core::{PlatformError, PlatformState, YamlResourceRepo};
use question_id::next_question_id;
use serde::{Deserialize, Serialize};

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
    /// Pagination metadata.
    pub pagination: gov_http_core::Pagination,
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
    #[serde(flatten)]
    pub pagination: gov_http_core::PaginationParams,
}

/// Request for creating a new question.
#[derive(Debug, Deserialize)]
pub struct CreateQuestionRequest {
    /// Category/component (e.g., TPL, BUNDLE).
    pub category: String,
    /// Brief summary.
    pub summary: String,
    /// Originating flow.
    pub flow: String,
    /// Phase within flow.
    pub phase: String,
    /// Detailed description.
    pub description: String,
    /// Who created it (human, agent, flow).
    pub created_by: String,
    /// Optional related task ID.
    pub task_id: Option<String>,
    /// Optional REQ/AC references.
    #[serde(default)]
    pub refs: Vec<String>,
}

/// Request for resolving a question.
#[derive(Debug, Deserialize)]
pub struct ResolveQuestionRequest {
    /// Who resolved it.
    pub resolved_by: String,
    /// Label of the chosen option (if any).
    pub chosen_option: Option<String>,
    /// Resolution notes.
    pub notes: Option<String>,
    /// New status (answered, resolved, obsolete).
    pub status: String,
}

/// Router for question endpoints.
///
/// Returns a router that handles:
/// - `GET /questions` - List all questions (with optional status filter)
/// - `POST /questions` - Create a new question
/// - `GET /questions/{id}` - Get a specific question
/// - `PUT /questions/{id}` - Resolve a question
/// - `DELETE /questions/{id}` - Delete a question
pub fn router<S>() -> Router<S>
where
    S: PlatformState + Clone + Send + Sync + 'static,
{
    Router::new().route("/questions", get(get_all_questions::<S>).post(post_question::<S>)).route(
        "/questions/{id}",
        get(get_question_by_id::<S>).put(put_question::<S>).delete(delete_question::<S>),
    )
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

    let response = tokio::task::spawn_blocking(move || {
        let repo = YamlResourceRepo::<Question>::new(&root, "questions");

        repo.list(
            filters.pagination,
            |q: &Question| {
                if let Some(ref status) = status_filter {
                    q.status.eq_ignore_ascii_case(status)
                } else {
                    true
                }
            },
            |a: &Question, b: &Question| b.created_at.cmp(&a.created_at), // Sort by created_at desc
        )
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    let summaries: Vec<QuestionSummary> = response
        .data
        .into_iter()
        .map(|q| QuestionSummary {
            id: q.id.clone(),
            summary: q.summary.clone(),
            status: q.status.clone(),
            flow: q.context.flow.clone(),
            phase: q.context.phase.clone(),
            created_at: q.created_at.clone(),
        })
        .collect();

    Ok(Json(QuestionsListResponse {
        total: response.pagination.total_items,
        questions: summaries,
        pagination: response.pagination,
    }))
}

/// GET /questions/{id} - Get a specific question entry by ID
async fn get_question_by_id<S>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<Question>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let question = tokio::task::spawn_blocking(move || {
        spec_runtime::validate_question_id(&id)
            .map_err(|e| PlatformError::internal(format!("Invalid ID format: {}", e)))?;

        let repo = YamlResourceRepo::<Question>::new(&root, "questions");
        repo.get(&id)
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(Json(question))
}

/// POST /questions - Create a new question.
async fn post_question<S>(
    State(state): State<S>,
    Json(req): Json<CreateQuestionRequest>,
) -> Result<(StatusCode, Json<Question>), PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let question = tokio::task::spawn_blocking(move || {
        // Validate inputs
        spec_runtime::validate_name("category", &req.category)
            .map_err(|e| PlatformError::internal(format!("Invalid category: {}", e)))?;

        let repo = YamlResourceRepo::<Question>::new(&root, "questions");

        // Get all to find next ID
        let existing = repo.list(
            gov_http_core::PaginationParams::default(),
            |_| true,
            |_, _| std::cmp::Ordering::Equal,
        )?;
        let id = next_question_id(&req.category, existing.data.iter().map(|q| q.id.as_str()));

        let question = Question {
            id,
            task_id: req.task_id,
            req_ids: Vec::new(),
            ac_ids: Vec::new(),
            refs: req.refs,
            summary: spec_runtime::sanitize_string(&req.summary),
            context: QuestionContext {
                flow: req.flow,
                phase: req.phase,
                description: Some(spec_runtime::sanitize_string(&req.description)),
                files_involved: Vec::new(),
            },
            options: Vec::new(),
            recommendation: None,
            created_by: req.created_by,
            created_at: Utc::now().to_rfc3339(),
            status: "open".to_string(),
            resolution: None,
        };

        repo.save(&question)?;
        Ok(question)
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok((StatusCode::CREATED, Json(question)))
}

/// PUT /questions/{id} - Resolve a question.
async fn put_question<S>(
    State(state): State<S>,
    Path(id): Path<String>,
    Json(req): Json<ResolveQuestionRequest>,
) -> Result<Json<Question>, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    let question = tokio::task::spawn_blocking(move || {
        spec_runtime::validate_question_id(&id)
            .map_err(|e| PlatformError::internal(format!("Invalid ID format: {}", e)))?;

        let repo = YamlResourceRepo::<Question>::new(&root, "questions");
        let mut question: Question = repo.get(&id)?;

        question.status = req.status;
        question.resolution = Some(QuestionResolution {
            resolved_by: req.resolved_by,
            resolved_at: Utc::now().to_rfc3339(),
            chosen_option: req.chosen_option.unwrap_or_default(),
            notes: req.notes.map(|n| spec_runtime::sanitize_string(&n)),
        });

        repo.save(&question)?;
        Ok(question)
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(Json(question))
}

/// DELETE /questions/{id} - Delete a question.
async fn delete_question<S>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<StatusCode, PlatformError>
where
    S: PlatformState,
{
    let root = state.context().root().to_path_buf();

    tokio::task::spawn_blocking(move || {
        spec_runtime::validate_question_id(&id)
            .map_err(|e| PlatformError::internal(format!("Invalid ID format: {}", e)))?;

        let repo = YamlResourceRepo::<Question>::new(&root, "questions");
        repo.delete(&id)
    })
    .await
    .map_err(|e| PlatformError::internal(format!("spawn_blocking failed: {}", e)))??;

    Ok(StatusCode::NO_CONTENT)
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
