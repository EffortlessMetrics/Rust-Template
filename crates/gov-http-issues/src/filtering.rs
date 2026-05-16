use crate::models::{
    Issue, IssueFilters, IssueKind, IssueStatus, IssuesSummary, KindCounts, StatusCounts,
};

pub(crate) fn apply_filters(issues: Vec<Issue>, filters: &IssueFilters) -> Vec<Issue> {
    issues.into_iter().filter(|issue| matches_filters(issue, filters)).collect()
}

fn matches_filters(issue: &Issue, filters: &IssueFilters) -> bool {
    matches_kind(issue, filters)
        && matches_status(issue, filters)
        && matches_priority(issue, filters)
        && matches_date_range(issue, filters)
        && matches_text(issue, filters)
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

fn matches_text(issue: &Issue, filters: &IssueFilters) -> bool {
    let Some(query) = &filters.q else {
        return true;
    };

    let query = query.to_lowercase();
    issue.id.to_lowercase().contains(&query)
        || issue.summary.to_lowercase().contains(&query)
        || issue.category.as_ref().is_some_and(|c| c.to_lowercase().contains(&query))
        || issue.labels.iter().any(|l| l.to_lowercase().contains(&query))
}

pub(crate) fn calculate_summary(issues: &[Issue]) -> IssuesSummary {
    let mut by_kind = KindCounts { friction: 0, question: 0, task: 0 };
    let mut by_status = StatusCounts { open: 0, in_progress: 0, resolved: 0 };

    for issue in issues {
        count_kind(&mut by_kind, issue.kind);
        count_status(&mut by_status, issue.status);
    }

    IssuesSummary { total: issues.len(), by_kind, by_status }
}

fn count_kind(counts: &mut KindCounts, kind: IssueKind) {
    match kind {
        IssueKind::Friction => counts.friction += 1,
        IssueKind::Question => counts.question += 1,
        IssueKind::Task => counts.task += 1,
    }
}

fn count_status(counts: &mut StatusCounts, status: IssueStatus) {
    match status {
        IssueStatus::Open => counts.open += 1,
        IssueStatus::InProgress => counts.in_progress += 1,
        IssueStatus::Resolved => counts.resolved += 1,
    }
}
