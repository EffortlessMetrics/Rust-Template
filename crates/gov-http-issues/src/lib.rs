//! Unified issues endpoint aggregating friction, questions, and tasks.
//!
//! This crate exposes the `/issues` router and keeps each responsibility in a
//! focused submodule: HTTP handling, YAML loading, issue conversion, filtering,
//! and public response models.
//!
//! # Endpoints
//!
//! - `GET /issues` - List all issues with filtering and pagination

mod conversion;
mod filtering;
mod handler;
mod loading;
mod models;

pub use handler::router;
pub use models::{
    Issue, IssueFilters, IssueKind, IssueStatus, IssuesResponse, IssuesSummary, KindCounts,
    Pagination, StatusCounts,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::extract_priority_from_labels;
    use crate::filtering::{apply_filters, calculate_summary};
    use gov_http_types::FrictionEntry;

    #[test]
    fn test_friction_to_issue_conversion() {
        let friction = FrictionEntry {
            id: "FRICTION-TEST-001".to_string(),
            date: "2025-11-26".to_string(),
            category: "tooling".to_string(),
            severity: "high".to_string(),
            summary: "Test friction".to_string(),
            description: "Test description".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "open".to_string(),
            resolution: None,
            refs: vec!["REQ-001".to_string()],
            related_items: None,
        };

        let issue: Issue = friction.into();
        assert_eq!(issue.id, "FRICTION-TEST-001");
        assert_eq!(issue.kind, IssueKind::Friction);
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.priority, 2); // high = 2
        assert_eq!(issue.category, Some("tooling".to_string()));
    }

    #[test]
    fn test_friction_resolved_status() {
        let friction = FrictionEntry {
            id: "FRICTION-TEST-002".to_string(),
            date: "2025-11-26".to_string(),
            category: "tooling".to_string(),
            severity: "medium".to_string(),
            summary: "Resolved friction".to_string(),
            description: "Test".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "resolved".to_string(),
            resolution: None,
            refs: vec![],
            related_items: None,
        };

        let issue: Issue = friction.into();
        assert_eq!(issue.status, IssueStatus::Resolved);
    }

    #[test]
    fn test_priority_from_labels() {
        assert_eq!(extract_priority_from_labels(&["p0".to_string()]), 1);
        assert_eq!(extract_priority_from_labels(&["p1".to_string()]), 2);
        assert_eq!(extract_priority_from_labels(&["P2".to_string()]), 3);
        assert_eq!(extract_priority_from_labels(&["p3".to_string()]), 4);
        assert_eq!(extract_priority_from_labels(&["other".to_string()]), 3);
        assert_eq!(extract_priority_from_labels(&[]), 3);
    }

    #[test]
    fn test_filter_by_kind() {
        let issues = vec![
            Issue {
                id: "FRICTION-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let filters = IssueFilters { kind: Some(IssueKind::Friction), ..Default::default() };

        let filtered = apply_filters(issues, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "FRICTION-001");
    }

    #[test]
    fn test_filter_by_text_search() {
        let issues = vec![
            Issue {
                id: "FRICTION-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Build failure in CI".to_string(),
                priority: 3,
                created_at: None,
                category: Some("ci_cd".to_string()),
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "API design question".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let filters = IssueFilters { q: Some("CI".to_string()), ..Default::default() };

        let filtered = apply_filters(issues, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "FRICTION-001");
    }

    #[test]
    fn test_summary_calculation() {
        let issues = vec![
            Issue {
                id: "F-001".to_string(),
                kind: IssueKind::Friction,
                status: IssueStatus::Open,
                native_status: "open".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "Q-001".to_string(),
                kind: IssueKind::Question,
                status: IssueStatus::InProgress,
                native_status: "answered".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
            Issue {
                id: "T-001".to_string(),
                kind: IssueKind::Task,
                status: IssueStatus::Resolved,
                native_status: "Done".to_string(),
                summary: "Test".to_string(),
                priority: 3,
                created_at: None,
                category: None,
                refs: vec![],
                owner: None,
                labels: vec![],
            },
        ];

        let summary = calculate_summary(&issues);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_kind.friction, 1);
        assert_eq!(summary.by_kind.question, 1);
        assert_eq!(summary.by_kind.task, 1);
        assert_eq!(summary.by_status.open, 1);
        assert_eq!(summary.by_status.in_progress, 1);
        assert_eq!(summary.by_status.resolved, 1);
    }
}
