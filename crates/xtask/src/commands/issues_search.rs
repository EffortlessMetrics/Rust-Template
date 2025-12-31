//! Cross-system issue search across friction, questions, and tasks.

use anyhow::Result;
use serde::Serialize;

use crate::commands::friction::{FrictionEntry, load_all_friction_entries};
use crate::commands::questions::{Question, load_all_questions};

/// Unified search result
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Issue type: friction, question, or task
    pub issue_type: String,
    /// Unique ID
    pub id: String,
    /// Summary/title
    pub summary: String,
    /// Current status
    pub status: String,
    /// Related REQ/AC refs
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    /// Date (created_at or date field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Relevance score (for sorting)
    pub relevance_score: f32,
}

/// JSON output structure
#[derive(Debug, Serialize)]
struct SearchOutput {
    query: String,
    total_results: usize,
    results: Vec<SearchResult>,
}

/// Search across friction, questions, and tasks.
pub fn search_issues(
    query: &str,
    type_filter: Option<&str>,
    status_filter: Option<&str>,
    refs_filter: Option<&str>,
    json: bool,
    limit: usize,
) -> Result<()> {
    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    // Search friction entries
    if type_filter.is_none() || type_filter == Some("friction") {
        let friction_entries = load_all_friction_entries().unwrap_or_default();
        for entry in friction_entries {
            if let Some(result) = search_friction(&entry, &query_lower, status_filter, refs_filter)
            {
                results.push(result);
            }
        }
    }

    // Search questions
    if type_filter.is_none() || type_filter == Some("question") {
        let questions = load_all_questions().unwrap_or_default();
        for question in questions {
            if let Some(result) =
                search_question(&question, &query_lower, status_filter, refs_filter)
            {
                results.push(result);
            }
        }
    }

    // Search tasks
    if (type_filter.is_none() || type_filter == Some("task"))
        && let Ok(tasks) = load_tasks()
    {
        for task in tasks {
            if let Some(result) = search_task(&task, &query_lower, status_filter, refs_filter) {
                results.push(result);
            }
        }
    }

    // Sort by relevance (highest first)
    results.sort_by(|a, b| {
        b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit results
    results.truncate(limit);

    if json {
        let output =
            SearchOutput { query: query.to_string(), total_results: results.len(), results };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if results.is_empty() {
            println!("No results found for '{}'", query);
            return Ok(());
        }

        println!("\nFound {} results for '{}':\n", results.len(), query);
        println!("{:<12} {:<24} {:<12} SUMMARY", "TYPE", "ID", "STATUS");
        println!("{}", "─".repeat(80));

        for result in &results {
            let summary_truncated = if result.summary.len() > 40 {
                format!("{}...", &result.summary[..37])
            } else {
                result.summary.clone()
            };
            println!(
                "{:<12} {:<24} {:<12} {}",
                result.issue_type, result.id, result.status, summary_truncated
            );
        }
        println!();
    }

    Ok(())
}

fn search_friction(
    entry: &FrictionEntry,
    query: &str,
    status_filter: Option<&str>,
    refs_filter: Option<&str>,
) -> Option<SearchResult> {
    // Apply status filter
    if let Some(filter) = status_filter
        && entry.status.to_lowercase() != filter.to_lowercase()
    {
        return None;
    }

    // Apply refs filter
    if let Some(filter) = refs_filter
        && !entry.refs.iter().any(|r| r.to_lowercase().contains(&filter.to_lowercase()))
    {
        return None;
    }

    // Calculate relevance
    let mut score = 0.0;

    // ID match (highest)
    if entry.id.to_lowercase().contains(query) {
        score += 10.0;
        if entry.id.to_lowercase() == query {
            score += 5.0; // Exact match bonus
        }
    }

    // Summary match
    if entry.summary.to_lowercase().contains(query) {
        score += 5.0;
    }

    // Description match
    if entry.description.to_lowercase().contains(query) {
        score += 2.0;
    }

    // Category match
    if entry.category.to_lowercase().contains(query) {
        score += 3.0;
    }

    if score > 0.0 {
        Some(SearchResult {
            issue_type: "friction".to_string(),
            id: entry.id.clone(),
            summary: entry.summary.clone(),
            status: entry.status.clone(),
            refs: entry.refs.clone(),
            date: Some(entry.date.clone()),
            relevance_score: score,
        })
    } else {
        None
    }
}

fn search_question(
    question: &Question,
    query: &str,
    status_filter: Option<&str>,
    refs_filter: Option<&str>,
) -> Option<SearchResult> {
    // Apply status filter
    if let Some(filter) = status_filter
        && question.status.to_lowercase() != filter.to_lowercase()
    {
        return None;
    }

    // Combine all refs
    let mut all_refs = question.req_ids.clone();
    all_refs.extend(question.ac_ids.clone());
    all_refs.extend(question.refs.clone());

    // Apply refs filter
    if let Some(filter) = refs_filter
        && !all_refs.iter().any(|r| r.to_lowercase().contains(&filter.to_lowercase()))
    {
        return None;
    }

    // Calculate relevance
    let mut score = 0.0;

    // ID match (highest)
    if question.id.to_lowercase().contains(query) {
        score += 10.0;
        if question.id.to_lowercase() == query {
            score += 5.0;
        }
    }

    // Summary match
    if question.summary.to_lowercase().contains(query) {
        score += 5.0;
    }

    // Context flow/phase match
    if question.context.flow.to_lowercase().contains(query) {
        score += 3.0;
    }
    if question.context.phase.to_lowercase().contains(query) {
        score += 2.0;
    }

    // Description match
    if question.context.description.to_lowercase().contains(query) {
        score += 2.0;
    }

    if score > 0.0 {
        Some(SearchResult {
            issue_type: "question".to_string(),
            id: question.id.clone(),
            summary: question.summary.clone(),
            status: question.status.clone(),
            refs: all_refs,
            date: Some(question.created_at.clone()),
            relevance_score: score,
        })
    } else {
        None
    }
}

/// Simple task representation for search
struct TaskForSearch {
    id: String,
    title: String,
    summary: String,
    status: String,
    requirement: String,
    acs: Vec<String>,
    labels: Vec<String>,
}

fn load_tasks() -> Result<Vec<TaskForSearch>> {
    use std::fs;
    use std::path::Path;

    let tasks_path = Path::new("specs/tasks.yaml");
    if !tasks_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(tasks_path)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let mut tasks = Vec::new();

    if let Some(task_list) = yaml.get("tasks").and_then(|t| t.as_sequence()) {
        for task in task_list {
            let id = task.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let title = task.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let summary =
                task.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let status = task.get("status").and_then(|v| v.as_str()).unwrap_or("Todo").to_string();
            let requirement =
                task.get("requirement").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let acs = task
                .get("acs")
                .and_then(|v| v.as_sequence())
                .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let labels = task
                .get("labels")
                .and_then(|v| v.as_sequence())
                .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            tasks.push(TaskForSearch { id, title, summary, status, requirement, acs, labels });
        }
    }

    Ok(tasks)
}

fn search_task(
    task: &TaskForSearch,
    query: &str,
    status_filter: Option<&str>,
    refs_filter: Option<&str>,
) -> Option<SearchResult> {
    // Apply status filter
    if let Some(filter) = status_filter
        && task.status.to_lowercase() != filter.to_lowercase()
    {
        return None;
    }

    // Build refs list
    let mut refs = vec![task.requirement.clone()];
    refs.extend(task.acs.clone());

    // Apply refs filter
    if let Some(filter) = refs_filter
        && !refs.iter().any(|r| r.to_lowercase().contains(&filter.to_lowercase()))
    {
        return None;
    }

    // Calculate relevance
    let mut score = 0.0;

    // ID match (highest)
    if task.id.to_lowercase().contains(query) {
        score += 10.0;
        if task.id.to_lowercase() == query {
            score += 5.0;
        }
    }

    // Title match
    if task.title.to_lowercase().contains(query) {
        score += 5.0;
    }

    // Summary match
    if task.summary.to_lowercase().contains(query) {
        score += 3.0;
    }

    // Labels match
    if task.labels.iter().any(|l| l.to_lowercase().contains(query)) {
        score += 2.0;
    }

    // Requirement match
    if task.requirement.to_lowercase().contains(query) {
        score += 2.0;
    }

    if score > 0.0 {
        Some(SearchResult {
            issue_type: "task".to_string(),
            id: task.id.clone(),
            summary: if task.summary.is_empty() {
                task.title.clone()
            } else {
                task.summary.clone()
            },
            status: task.status.clone(),
            refs,
            date: None,
            relevance_score: score,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            issue_type: "friction".to_string(),
            id: "FRICTION-TEST-001".to_string(),
            summary: "Test friction".to_string(),
            status: "open".to_string(),
            refs: vec!["REQ-001".to_string()],
            date: Some("2025-01-01".to_string()),
            relevance_score: 10.0,
        };

        let json = serde_json::to_string(&result).expect("should serialize");
        assert!(json.contains("FRICTION-TEST-001"));
        assert!(json.contains("friction"));
    }
}
