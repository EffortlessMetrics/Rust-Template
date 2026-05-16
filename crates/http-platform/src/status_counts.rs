use platform_contract::{
    ForkCounts, FrictionCounts, FrictionSummary, QuestionBrief, QuestionCounts, SeverityCounts,
    TaskStatusBreakdown,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Calculate task counts broken down by status.
pub(super) fn calculate_task_breakdown(
    tasks_spec: &spec_runtime::TasksSpec,
) -> TaskStatusBreakdown {
    let mut breakdown = TaskStatusBreakdown::new(0, 0, 0, 0);
    for task in &tasks_spec.tasks {
        let status = &task.status;
        match status.to_lowercase().as_str() {
            "open" | "todo" => breakdown.todo += 1,
            "inprogress" | "in_progress" | "in-progress" => breakdown.in_progress += 1,
            "review" => breakdown.review += 1,
            "done" | "closed" => breakdown.done += 1,
            _ => breakdown.todo += 1,
        }
    }
    breakdown
}

/// Load question counts from questions/ directory.
pub(super) fn load_question_counts(root: &Path) -> QuestionCounts {
    #[derive(Deserialize)]
    struct Question {
        pub id: String,
        #[serde(default)]
        pub summary: String,
        #[serde(default)]
        pub status: String,
        pub context: QuestionContext,
    }

    #[derive(Deserialize)]
    struct QuestionContext {
        pub flow: String,
    }

    let questions_dir = root.join("questions");
    if !questions_dir.exists() {
        return QuestionCounts::new(0, 0, 0, 0, vec![]);
    }

    let mut open = 0;
    let mut answered = 0;
    let mut resolved = 0;
    let mut total = 0;
    let mut open_questions = Vec::new();

    if let Ok(entries) = fs::read_dir(&questions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(question) = serde_yaml::from_str::<Question>(&content)
            {
                total += 1;
                match question.status.as_str() {
                    "open" => {
                        open += 1;
                        open_questions.push(QuestionBrief::new(
                            question.id,
                            question.summary,
                            question.context.flow,
                        ));
                    }
                    "answered" => answered += 1,
                    "resolved" => resolved += 1,
                    _ => {}
                }
            }
        }
    }

    open_questions.truncate(3);

    QuestionCounts::new(open, answered, resolved, total, open_questions)
}

/// Load friction counts from friction/ directory.
pub(super) fn load_friction_counts(root: &Path) -> FrictionCounts {
    #[derive(Deserialize)]
    struct FrictionEntry {
        pub id: String,
        pub date: String,
        #[serde(default)]
        pub severity: String,
        #[serde(default)]
        pub summary: String,
        #[serde(default)]
        pub category: String,
        #[serde(default)]
        pub status: String,
    }

    let friction_dir = root.join("friction");
    if !friction_dir.exists() {
        return FrictionCounts::new(0, 0, SeverityCounts::new(0, 0, 0, 0), vec![]);
    }

    let mut total = 0;
    let mut open = 0;
    let mut by_severity = SeverityCounts::new(0, 0, 0, 0);
    let mut all_entries = Vec::new();

    if let Ok(entries) = fs::read_dir(&friction_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(friction) = serde_yaml::from_str::<FrictionEntry>(&content)
            {
                total += 1;

                if friction.status == "open" || friction.status.is_empty() {
                    open += 1;
                }

                match friction.severity.as_str() {
                    "low" => by_severity.low += 1,
                    "medium" => by_severity.medium += 1,
                    "high" => by_severity.high += 1,
                    "critical" => by_severity.critical += 1,
                    _ => {}
                }

                all_entries.push(friction);
            }
        }
    }

    all_entries.sort_by(|a, b| b.date.cmp(&a.date));
    let recent: Vec<FrictionSummary> = all_entries
        .into_iter()
        .take(5)
        .map(|e| FrictionSummary::new(e.id, e.date, e.severity, e.summary, e.category))
        .collect();

    FrictionCounts::new(total, open, by_severity, recent)
}

/// Load fork counts from forks/fork_registry.yaml.
pub(super) fn load_fork_counts(root: &Path) -> ForkCounts {
    #[derive(Deserialize)]
    struct ForkRegistry {
        #[serde(default)]
        pub forks: Vec<ForkEntry>,
    }

    #[derive(Deserialize)]
    struct ForkEntry {
        pub id: String,
    }

    let registry_path = root.join("forks/fork_registry.yaml");
    if !registry_path.exists() {
        return ForkCounts::new(0, vec![]);
    }

    if let Ok(content) = fs::read_to_string(&registry_path)
        && let Ok(registry) = serde_yaml::from_str::<ForkRegistry>(&content)
    {
        let ids: Vec<String> = registry.forks.iter().map(|f| f.id.clone()).collect();
        let total = ids.len();
        ForkCounts::new(total, ids)
    } else {
        ForkCounts::new(0, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_task_breakdown() {
        let tasks_spec = spec_runtime::TasksSpec {
            schema_version: "1.0".to_string(),
            template_version: "0.0.0".to_string(),
            tasks: vec![
                spec_runtime::Task {
                    id: "task-1".to_string(),
                    title: "Task 1".to_string(),
                    status: "todo".to_string(),
                    requirement: "REQ-001".to_string(),
                    acs: vec![],
                    labels: vec![],
                    owner: None,
                    docs: None,
                    summary: String::new(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
                spec_runtime::Task {
                    id: "task-2".to_string(),
                    title: "Task 2".to_string(),
                    status: "in_progress".to_string(),
                    requirement: "REQ-002".to_string(),
                    acs: vec![],
                    labels: vec![],
                    owner: None,
                    docs: None,
                    summary: String::new(),
                    recommended_flows: vec![],
                    depends_on: vec![],
                },
            ],
        };

        let breakdown = calculate_task_breakdown(&tasks_spec);
        assert_eq!(breakdown.todo, 1);
        assert_eq!(breakdown.in_progress, 1);
        assert_eq!(breakdown.review, 0);
        assert_eq!(breakdown.done, 0);
    }
}
