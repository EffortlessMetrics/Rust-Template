# gov-http-issues

Unified issues endpoint aggregating friction, questions, and tasks.

## Purpose

This crate provides a single `/issues` endpoint that normalizes all three artifact types (friction, questions, tasks) into a common `Issue` representation with unified status, priority, and filtering capabilities.

## Endpoints

- `GET /issues` - List all issues with filtering and pagination

## Dependencies

- `axum` - Web framework
- `http` - HTTP types
- `http-errors` - Error types and mapping (with axum feature)
- `platform-contract` - Platform contract types
- `gov-model` - Governance domain model
- `gov-http-core` - Shared router foundation
- `gov-http-friction` - Friction types (for conversion)
- `gov-http-questions` - Question types (for conversion)
- `spec-runtime` - Task spec loading
- `serde_yaml` - YAML parsing
- `tokio` - Async runtime

## Usage

```rust
use gov_http_issues::router;
use axum::Router;
use gov_http_core::PlatformState;

// Compose issues router
let app = Router::new()
    .merge(router::<MyState>())
    .with_state(my_state);
```

## Data Types

### Issue

Unified issue representation:
- `id`: Unique identifier (FRICTION-XXX, Q-XXX, TASK-XXX)
- `kind`: Issue type (friction, question, task)
- `status`: Normalized status (open, in_progress, resolved)
- `native_status`: Original status string from source
- `summary`: One-line summary
- `priority`: Priority level (1=critical/p0, 2=high/p1, 3=medium/p2, 4=low/p3)
- `created_at`: Creation/discovery date (nullable for tasks)
- `category`: Category or flow context
- `refs`: Related REQ/AC IDs
- `owner`: Owner/assignee
- `labels`: Labels/tags

### IssueKind

Issue type discriminator:
- `Friction` - DevEx friction entries
- `Question` - Design decision questions
- `Task` - Governance tasks

### IssueStatus

Normalized status across all issue types:
- `Open` - Not started
- `InProgress` - In progress
- `Resolved` - Completed

### IssueFilters

Query parameters for filtering:
- `kind`: Filter by issue kind
- `status`: Filter by status
- `priority`: Filter by exact priority (1-4)
- `min_priority`: Filter by minimum priority
- `from_date`: Filter by date range start
- `to_date`: Filter by date range end
- `q`: Text search in id, summary, category
- `page`: Page number (1-indexed, default 1)
- `per_page`: Items per page (default 50, max 100)

### IssuesResponse

Response containing:
- `issues`: Paginated list of issues
- `pagination`: Pagination metadata
- `summary`: Summary counts by kind and status

## Priority Mapping

Priority is derived differently per issue type:

- **Friction**: From severity field (critical=1, high=2, medium=3, low=4)
- **Questions**: Default medium (3)
- **Tasks**: From labels (p0=1, p1=2, p2=3, p3=4)

## Status Normalization

Status is normalized across issue types:

| Source Type | Source Status | Normalized Status |
|------------|---------------|-------------------|
| Friction | open | Open |
| Friction | investigating/in_progress | InProgress |
| Friction | resolved/wont_fix | Resolved |
| Question | open | Open |
| Question | answered | InProgress |
| Question | resolved/obsolete | Resolved |
| Task | open/todo | Open |
| Task | inprogress/in_progress/review | InProgress |
| Task | done/closed/complete | Resolved |
