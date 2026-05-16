use serde::{Deserialize, Serialize};

/// Issue type discriminator
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    Friction,
    Question,
    Task,
}

impl std::fmt::Display for IssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueKind::Friction => write!(f, "friction"),
            IssueKind::Question => write!(f, "question"),
            IssueKind::Task => write!(f, "task"),
        }
    }
}

/// Normalized status across all issue types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    InProgress,
    Resolved,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Open => write!(f, "open"),
            IssueStatus::InProgress => write!(f, "in_progress"),
            IssueStatus::Resolved => write!(f, "resolved"),
        }
    }
}

/// Unified issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier (FRICTION-XXX, Q-XXX, TASK-XXX)
    pub id: String,
    /// Issue type
    pub kind: IssueKind,
    /// Normalized status
    pub status: IssueStatus,
    /// Original/native status string (for transparency)
    pub native_status: String,
    /// One-line summary
    pub summary: String,
    /// Priority (1=critical/p0, 2=high/p1, 3=medium/p2, 4=low/p3)
    pub priority: u8,
    /// Creation/discovery date (ISO 8601, nullable for tasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Category or flow context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Related REQ/AC IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    /// Owner/assignee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Labels/tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Query parameters for filtering issues
#[derive(Debug, Default, Deserialize)]
pub struct IssueFilters {
    /// Filter by issue kind (friction, question, task)
    pub kind: Option<IssueKind>,
    /// Filter by status (open, in_progress, resolved)
    pub status: Option<IssueStatus>,
    /// Filter by priority (1-4)
    pub priority: Option<u8>,
    /// Filter by minimum priority (inclusive, 1=highest)
    pub min_priority: Option<u8>,
    /// Filter by date range start (ISO 8601)
    pub from_date: Option<String>,
    /// Filter by date range end (ISO 8601)
    pub to_date: Option<String>,
    /// Text search in id, summary, category
    pub q: Option<String>,
    /// Page number (1-indexed, default 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page (default 50, max 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Response for /platform/issues
#[derive(Debug, Serialize)]
pub struct IssuesResponse {
    pub issues: Vec<Issue>,
    pub pagination: Pagination,
    pub summary: IssuesSummary,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: usize,
    pub total_pages: u32,
}

/// Summary counts by kind and status
#[derive(Debug, Serialize)]
pub struct IssuesSummary {
    pub total: usize,
    pub by_kind: KindCounts,
    pub by_status: StatusCounts,
}

#[derive(Debug, Serialize)]
pub struct KindCounts {
    pub friction: usize,
    pub question: usize,
    pub task: usize,
}

#[derive(Debug, Serialize)]
pub struct StatusCounts {
    pub open: usize,
    pub in_progress: usize,
    pub resolved: usize,
}
