//! Filtering, sorting, pagination, and summary helpers for issue lists.

use crate::{Issue, IssueFilters, IssueKind, IssuesSummary, KindCounts, StatusCounts};

pub(crate) fn apply_filters(issues: Vec<Issue>, filters: &IssueFilters) -> Vec<Issue> {
    issues.into_iter().filter(|issue| matches_filters(issue, filters)).collect()
}

fn matches_filters(issue: &Issue, filters: &IssueFilters) -> bool {
    matches_kind(issue, filters)
        && matches_status(issue, filters)
        && matches_priority(issue, filters)
        && matches_date_range(issue, filters)
        && matches_text_search(issue, filters)
}

fn matches_kind(issue: &Issue, filters: &IssueFilters) -> bool {
    filters.kind.as_ref().is_none_or(|kind| &issue.kind == kind)
}

fn matches_status(issue: &Issue, filters: &IssueFilters) -> bool {
    filters.status.as_ref().is_none_or(|status| &issue.status == status)
}

fn matches_priority(issue: &Issue, filters: &IssueFilters) -> bool {
    if let Some(priority) = filters.priority
        && issue.priority != priority
    {
        return false;
    }

    // Min priority filter (1=highest, so <= comparison)
    if let Some(min_priority) = filters.min_priority
        && issue.priority > min_priority
    {
        return false;
    }

    true
}

fn matches_date_range(issue: &Issue, filters: &IssueFilters) -> bool {
    if let Some(from) = &filters.from_date
        && let Some(created) = &issue.created_at
        && created < from
    {
        return false;
    }

    if let Some(to) = &filters.to_date
        && let Some(created) = &issue.created_at
        && created > to
    {
        return false;
    }

    true
}

fn matches_text_search(issue: &Issue, filters: &IssueFilters) -> bool {
    let Some(query) = &filters.q else {
        return true;
    };

    let query = query.to_lowercase();
    issue.id.to_lowercase().contains(&query)
        || issue.summary.to_lowercase().contains(&query)
        || issue.category.as_ref().is_some_and(|category| category.to_lowercase().contains(&query))
        || issue.labels.iter().any(|label| label.to_lowercase().contains(&query))
}

pub(crate) fn calculate_summary(issues: &[Issue]) -> IssuesSummary {
    let mut by_kind = KindCounts { friction: 0, question: 0, task: 0 };
    let mut by_status = StatusCounts { open: 0, in_progress: 0, resolved: 0 };

    for issue in issues {
        match issue.kind {
            IssueKind::Friction => by_kind.friction += 1,
            IssueKind::Question => by_kind.question += 1,
            IssueKind::Task => by_kind.task += 1,
        }
        match issue.status {
            crate::IssueStatus::Open => by_status.open += 1,
            crate::IssueStatus::InProgress => by_status.in_progress += 1,
            crate::IssueStatus::Resolved => by_status.resolved += 1,
        }
    }

    IssuesSummary { total: issues.len(), by_kind, by_status }
}

pub(crate) fn sort_issues(issues: &mut [Issue]) {
    // Sort by priority (highest first), then by date (most recent first), then by ID (for determinism)
    issues.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests {
    use crate::{IssueStatus, Pagination};

    use super::*;

    fn test_issue(id: &str, kind: IssueKind, status: IssueStatus) -> Issue {
        Issue {
            id: id.to_string(),
            kind,
            status,
            native_status: "open".to_string(),
            summary: "Test".to_string(),
            priority: 3,
            created_at: None,
            category: None,
            refs: vec![],
            owner: None,
            labels: vec![],
        }
    }

    #[test]
    fn filter_by_kind() {
        let issues = vec![
            test_issue("FRICTION-001", IssueKind::Friction, IssueStatus::Open),
            test_issue("Q-001", IssueKind::Question, IssueStatus::Open),
        ];

        let filters = IssueFilters { kind: Some(IssueKind::Friction), ..Default::default() };

        let filtered = apply_filters(issues, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "FRICTION-001");
    }

    #[test]
    fn filter_by_text_search() {
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
    fn summary_calculation() {
        let issues = vec![
            test_issue("F-001", IssueKind::Friction, IssueStatus::Open),
            test_issue("Q-001", IssueKind::Question, IssueStatus::InProgress),
            test_issue("T-001", IssueKind::Task, IssueStatus::Resolved),
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

    #[test]
    fn pagination_type_stays_available_to_handler() {
        let pagination = Pagination { page: 1, per_page: 50, total_items: 0, total_pages: 0 };
        assert_eq!(pagination.page, 1);
    }
}
