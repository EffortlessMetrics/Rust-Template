//! Question types for tracking design decisions and ambiguities.

use serde::{Deserialize, Serialize};

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

/// Context about the question's origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    pub flow: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
}

/// An option/choice for a question
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

/// Recommendation for a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub option_label: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

/// Resolution details for a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResolution {
    pub resolved_by: String,
    pub resolved_at: String,
    pub chosen_option: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
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
}
