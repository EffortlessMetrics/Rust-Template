use gov_http_types::{FrictionEntry, Question};

use crate::models::{Issue, IssueKind, IssueStatus};

impl From<FrictionEntry> for Issue {
    fn from(f: FrictionEntry) -> Self {
        let status = match f.status.as_str() {
            "resolved" | "wont_fix" => IssueStatus::Resolved,
            "investigating" | "in_progress" => IssueStatus::InProgress,
            _ => IssueStatus::Open,
        };
        let priority = match f.severity.as_str() {
            "critical" => 1,
            "high" => 2,
            "medium" => 3,
            "low" => 4,
            _ => 3,
        };
        Issue {
            id: f.id,
            kind: IssueKind::Friction,
            status,
            native_status: f.status,
            summary: f.summary,
            priority,
            created_at: Some(f.date),
            category: Some(f.category),
            refs: f.refs,
            owner: f.context.as_ref().and_then(|c| c.discovered_by.clone()),
            labels: vec![],
        }
    }
}

impl From<Question> for Issue {
    fn from(q: Question) -> Self {
        let status = match q.status.as_str() {
            "resolved" | "obsolete" => IssueStatus::Resolved,
            "answered" => IssueStatus::InProgress,
            _ => IssueStatus::Open,
        };
        let mut refs = q.req_ids.clone();
        refs.extend(q.ac_ids.clone());
        refs.extend(q.refs.clone());
        Issue {
            id: q.id,
            kind: IssueKind::Question,
            status,
            native_status: q.status,
            summary: q.summary,
            priority: 3, // Default medium for questions
            created_at: Some(q.created_at),
            category: Some(q.context.flow),
            refs,
            owner: Some(q.created_by),
            labels: vec![],
        }
    }
}

/// Convert a task to an Issue.
pub(crate) fn task_to_issue(task: &spec_runtime::Task, effective_status: &str) -> Issue {
    let status = match effective_status.to_lowercase().as_str() {
        "done" => IssueStatus::Resolved,
        "inprogress" | "in_progress" | "review" => IssueStatus::InProgress,
        _ => IssueStatus::Open,
    };

    let priority = extract_priority_from_labels(&task.labels);

    let mut refs = vec![task.requirement.clone()];
    refs.extend(task.acs.iter().cloned());

    Issue {
        id: task.id.clone(),
        kind: IssueKind::Task,
        status,
        native_status: effective_status.to_string(),
        summary: task.summary.clone(),
        priority,
        created_at: None, // Tasks don't have creation date in spec
        category: Some(task.requirement.clone()),
        refs,
        owner: task.owner.clone(),
        labels: task.labels.clone(),
    }
}

pub(crate) fn extract_priority_from_labels(labels: &[String]) -> u8 {
    for label in labels {
        match label.to_lowercase().as_str() {
            "p0" => return 1,
            "p1" => return 2,
            "p2" => return 3,
            "p3" => return 4,
            _ => {}
        }
    }
    3 // Default medium
}
