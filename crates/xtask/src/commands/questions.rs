use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Question artifact representing ambiguity encountered by flows
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
    pub resolution: Option<Resolution>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    pub flow: String,
    pub phase: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reversible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub option_label: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub resolved_by: String,
    pub resolved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_option: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Question statistics
#[derive(Debug, Default)]
pub struct QuestionStats {
    pub open_count: usize,
    pub answered_count: usize,
    pub resolved_count: usize,
    pub obsolete_count: usize,
    pub total_count: usize,
}

impl Question {
    /// Create a new question with current timestamp
    #[allow(dead_code)]
    pub fn new(
        id: String,
        flow: &str,
        phase: &str,
        summary: String,
        description: String,
        created_by: &str,
    ) -> Self {
        let now: DateTime<Utc> = Utc::now();
        Question {
            id,
            task_id: None,
            req_ids: Vec::new(),
            ac_ids: Vec::new(),
            refs: Vec::new(),
            summary,
            context: QuestionContext {
                flow: flow.to_string(),
                phase: phase.to_string(),
                description,
                files_involved: Vec::new(),
            },
            options: Vec::new(),
            recommendation: None,
            created_by: created_by.to_string(),
            created_at: now.to_rfc3339(),
            status: "open".to_string(),
            resolution: None,
        }
    }

    /// Save question to YAML file in questions/ directory
    #[allow(dead_code)]
    pub fn save(&self) -> Result<PathBuf> {
        let questions_dir = Path::new("questions");
        fs::create_dir_all(questions_dir)
            .with_context(|| format!("Failed to create directory: {}", questions_dir.display()))?;

        let filename = format!("{}.yaml", self.id);
        let filepath = questions_dir.join(&filename);

        let yaml_content = serde_yaml::to_string(&self)
            .with_context(|| format!("Failed to serialize question: {}", self.id))?;

        // Add header comment
        let content = format!(
            "# Question: {}\n# Created by {} at {}\n# Status: {}\n\n{}",
            self.summary, self.created_by, self.created_at, self.status, yaml_content
        );

        fs::write(&filepath, content)
            .with_context(|| format!("Failed to write question file: {}", filepath.display()))?;

        Ok(filepath)
    }

    /// Load question from YAML file
    pub fn load(filepath: &Path) -> Result<Self> {
        let content = fs::read_to_string(filepath)
            .with_context(|| format!("Failed to read question file: {}", filepath.display()))?;

        // Parse YAML, ignoring comment lines
        let question: Question = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse question YAML: {}", filepath.display()))?;

        Ok(question)
    }
}

/// Load all questions from questions/ directory
pub fn load_all_questions() -> Result<Vec<Question>> {
    let questions_dir = Path::new("questions");
    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    for entry in fs::read_dir(questions_dir)
        .with_context(|| format!("Failed to read directory: {}", questions_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match Question::load(&path) {
            Ok(question) => questions.push(question),
            Err(e) => {
                eprintln!("Warning: Failed to load question from {}: {}", path.display(), e);
            }
        }
    }

    Ok(questions)
}

/// Calculate question statistics
pub fn calculate_stats(questions: &[Question]) -> QuestionStats {
    let mut stats = QuestionStats::default();

    for question in questions {
        stats.total_count += 1;
        match question.status.as_str() {
            "open" => stats.open_count += 1,
            "answered" => stats.answered_count += 1,
            "resolved" => stats.resolved_count += 1,
            "obsolete" => stats.obsolete_count += 1,
            _ => {}
        }
    }

    stats
}

/// JSON output structure for questions-list
#[derive(Debug, Serialize)]
struct QuestionsListJson {
    timestamp: String,
    total_count: usize,
    stats: QuestionStatsJson,
    questions: Vec<Question>,
}

#[derive(Debug, Serialize)]
struct QuestionStatsJson {
    open: usize,
    answered: usize,
    resolved: usize,
    obsolete: usize,
}

/// List questions filtered by status
#[allow(dead_code)]
pub fn list_questions(status_filter: Option<&str>, json: bool) -> Result<()> {
    let questions = load_all_questions()?;

    let filtered: Vec<&Question> = match status_filter {
        Some(filter) => questions.iter().filter(|q| q.status == filter).collect(),
        None => questions.iter().collect(),
    };

    if filtered.is_empty() {
        if json {
            // Empty JSON output
            let output = QuestionsListJson {
                timestamp: chrono::Utc::now().to_rfc3339(),
                total_count: 0,
                stats: QuestionStatsJson { open: 0, answered: 0, resolved: 0, obsolete: 0 },
                questions: Vec::new(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No questions found.");
        }
        return Ok(());
    }

    let stats = calculate_stats(&questions);

    if json {
        // JSON output
        let output = QuestionsListJson {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_count: filtered.len(),
            stats: QuestionStatsJson {
                open: stats.open_count,
                answered: stats.answered_count,
                resolved: stats.resolved_count,
                obsolete: stats.obsolete_count,
            },
            questions: filtered.into_iter().cloned().collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        println!("\n{} Questions:\n", if status_filter.is_some() { "Filtered" } else { "All" });

        for question in filtered {
            let status_badge = match question.status.as_str() {
                "open" => "⚠️  OPEN",
                "answered" => "💬 ANSWERED",
                "resolved" => "✅ RESOLVED",
                "obsolete" => "🗑️  OBSOLETE",
                _ => "❓ UNKNOWN",
            };

            println!("  {} {} - {}", status_badge, question.id, question.summary);
            println!("     Flow: {} / {}", question.context.flow, question.context.phase);
            if let Some(task_id) = &question.task_id {
                println!("     Task: {}", task_id);
            }
            if !question.ac_ids.is_empty() {
                println!("     ACs: {}", question.ac_ids.join(", "));
            }
            println!();
        }

        println!(
            "Total: {} (open: {}, answered: {}, resolved: {}, obsolete: {})\n",
            stats.total_count,
            stats.open_count,
            stats.answered_count,
            stats.resolved_count,
            stats.obsolete_count
        );
    }

    Ok(())
}

/// Get next question ID for a given flow category
#[allow(dead_code)]
pub fn get_next_question_id(category: &str) -> Result<usize> {
    let questions = load_all_questions()?;

    // Find highest ID number for this category
    let prefix = format!("Q-{}-", category.to_uppercase());
    let max_id = questions
        .iter()
        .filter(|q| q.id.starts_with(&prefix))
        .filter_map(|q| q.id.strip_prefix(&prefix).and_then(|s| s.parse::<usize>().ok()))
        .max()
        .unwrap_or(0);

    Ok(max_id + 1)
}

/// Emit a question artifact from a flow
#[allow(dead_code)]
pub fn emit_question(question: Question) -> Result<()> {
    let filepath = question.save()?;
    eprintln!("⚠️  Question {} created: {}", question.id, question.summary);
    eprintln!("   File: {}", filepath.display());
    Ok(())
}

/// Create a new question artifact via CLI
#[allow(clippy::too_many_arguments)]
pub fn create_question(
    category: &str,
    summary: &str,
    flow: &str,
    phase: &str,
    description: &str,
    created_by: &str,
    task_id: Option<&str>,
    refs: &[String],
) -> Result<()> {
    // Validate category format (uppercase alphanumeric)
    let category_upper = category.to_uppercase();
    if !category_upper.chars().all(|c| c.is_ascii_alphanumeric()) {
        anyhow::bail!(
            "Invalid category '{}'. Category must be alphanumeric (e.g., TPL, BUNDLE, SUGGEST)",
            category
        );
    }

    // Validate created_by
    let valid_created_by = ["agent", "human", "flow"];
    if !valid_created_by.contains(&created_by) {
        anyhow::bail!(
            "Invalid created_by '{}'. Must be one of: {}",
            created_by,
            valid_created_by.join(", ")
        );
    }

    // Generate next question ID
    let next_id = get_next_question_id(&category_upper)?;
    let id = format!("Q-{}-{:03}", category_upper, next_id);

    // Create question
    let mut question = Question::new(
        id.clone(),
        flow,
        phase,
        summary.to_string(),
        description.to_string(),
        created_by,
    );

    // Add task_id if provided
    if let Some(tid) = task_id {
        question.task_id = Some(tid.to_string());
    }

    // Add refs if provided
    question.refs = refs.to_vec();

    // Save to file
    let filepath = question.save()?;

    println!("✅ Created question: {}", id);
    println!("   File: {}", filepath.display());
    println!("   Flow: {} / {}", flow, phase);
    println!("   Created by: {}", created_by);
    if let Some(tid) = task_id {
        println!("   Task: {}", tid);
    }
    println!("   Status: open");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_serialization() {
        let question = Question::new(
            "Q-TEST-001".to_string(),
            "bundle",
            "ac_selection",
            "Test question summary".to_string(),
            "Test question description".to_string(),
            "flow",
        );

        let yaml = serde_yaml::to_string(&question).unwrap();
        assert!(yaml.contains("Q-TEST-001"));
        assert!(yaml.contains("bundle"));
        assert!(yaml.contains("ac_selection"));
    }

    #[test]
    fn test_question_deserialization() {
        let yaml = r#"
id: Q-TEST-002
summary: "Test question"
context:
  flow: suggest-next
  phase: dependency_analysis
  description: "Test description"
created_by: flow
created_at: "2025-11-26T00:00:00Z"
status: open
"#;

        let question: Question = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(question.id, "Q-TEST-002");
        assert_eq!(question.context.flow, "suggest-next");
        assert_eq!(question.status, "open");
    }

    #[test]
    fn artifacts_have_refs() {
        // Test Question struct has refs field that can hold REQ-*/AC-* references
        let yaml = r#"
id: Q-TEST-003
summary: "Test question with refs"
context:
  flow: bundle
  phase: validation
  description: "Testing refs field"
created_by: flow
created_at: "2025-11-26T00:00:00Z"
status: open
refs:
  - REQ-TPL-GOV-ARTIFACTS
  - AC-TPL-ARTIFACTS-HAVE-REFS
"#;

        let question: Question = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(question.id, "Q-TEST-003");
        assert_eq!(question.refs.len(), 2);
        assert_eq!(question.refs[0], "REQ-TPL-GOV-ARTIFACTS");
        assert_eq!(question.refs[1], "AC-TPL-ARTIFACTS-HAVE-REFS");

        // Test serialization round-trip
        let serialized = serde_yaml::to_string(&question).unwrap();
        assert!(serialized.contains("REQ-TPL-GOV-ARTIFACTS"));
        assert!(serialized.contains("AC-TPL-ARTIFACTS-HAVE-REFS"));

        // Test FrictionEntry struct has refs field (imported from friction module)
        use crate::commands::friction::FrictionEntry;

        let friction_yaml = r#"
id: FRICTION-TEST-003
date: "2025-11-26"
category: governance
severity: low
summary: "Test friction with refs"
description: "Testing refs field on friction"
status: open
refs:
  - REQ-TPL-GOV-ARTIFACTS
  - AC-TPL-ARTIFACTS-HAVE-REFS
"#;

        let friction: FrictionEntry = serde_yaml::from_str(friction_yaml).unwrap();
        assert_eq!(friction.id, "FRICTION-TEST-003");
        assert_eq!(friction.refs.len(), 2);
        assert_eq!(friction.refs[0], "REQ-TPL-GOV-ARTIFACTS");
        assert_eq!(friction.refs[1], "AC-TPL-ARTIFACTS-HAVE-REFS");

        // Test serialization round-trip
        let serialized = serde_yaml::to_string(&friction).unwrap();
        assert!(serialized.contains("REQ-TPL-GOV-ARTIFACTS"));
        assert!(serialized.contains("AC-TPL-ARTIFACTS-HAVE-REFS"));
    }
}
