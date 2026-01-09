---
id: DESIGN-PLT-ISSUES-ENDPOINT-001
title: Unified Issues Endpoint
author: system
doc_type: design_doc
date: 2026-01-07
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ISSUES-ENDPOINT]
tags: [platform, governance, api]
acs: [AC-GOV-025]
adrs: []
---

# Unified Issues Endpoint

## Context

The platform cell generates multiple types of governance artifacts that need attention:
- **Friction entries** (`friction/*.yaml`) — DevEx issues and process problems
- **Questions** — Design ambiguities captured during development
- **Tasks** (`specs/tasks.yaml`) — Work items with status and dependencies

Previously, consumers (agents, IDPs, reviewers) had to query multiple endpoints and merge results manually:
- `/platform/friction` for friction entries
- `/platform/questions` for design questions
- `/platform/tasks` for work items

This created unnecessary complexity for common use cases like "show me all open issues" or "what needs attention next."

## Decision

Provide a unified `/platform/issues` endpoint that consolidates all governance artifacts into a single, paginated, filterable view with stable ordering.

### Endpoint Contract

**GET /platform/issues**

Returns a paginated list of issues from all sources with:

- **Stable schema** — consistent `IssueKind` enum (`Friction`, `Question`, `Task`)
- **Deterministic ordering** — priority DESC, date DESC, id ASC
- **Filtering** — by `kind`, `status`, `text` (search)
- **Cursor-based pagination** — supports `cursor` and `limit` params

**Response Shape:**

```json
{
  "issues": [
    {
      "id": "FRICTION-001",
      "kind": "Friction",
      "title": "...",
      "status": "Open",
      "priority": 2,
      "created_at": "2026-01-01T00:00:00Z",
      "source": "friction/FRICTION-001.yaml"
    }
  ],
  "pagination": {
    "next_cursor": "...",
    "has_more": true
  },
  "summary": {
    "total": 42,
    "by_kind": { "Friction": 10, "Question": 12, "Task": 20 },
    "by_status": { "Open": 30, "InProgress": 8, "Closed": 4 }
  }
}
```

### Error Semantics

The endpoint returns HTTP 400 with descriptive messages for:

- **Mixed pagination params** — cannot use both `cursor` and `offset`
- **Invalid cursor** — cursor string is malformed or corrupted
- **Oversized cursor** — cursor exceeds maximum allowed length
- **Unknown cursor version** — cursor version not recognized

This explicit error handling ensures consumers can distinguish between "no results" (200 with empty array) and "bad request" (400).

## Implementation

### Data Model

The `IssueKind` enum normalizes source types:

```rust
pub enum IssueKind {
    Friction,
    Question,
    Task,
}

pub enum IssueStatus {
    Open,
    InProgress,
    Closed,
    Resolved,
}
```

### Ordering Contract

Issues are sorted deterministically by:
1. Priority (higher first)
2. Created date (newer first)
3. ID (alphabetically, for ties)

This ensures cursor-based pagination produces stable results even as the underlying data changes.

### Pagination Model

Cursor-based pagination with versioned cursors:
- Cursor encodes: `(version, offset, ordering_key)`
- Version allows schema evolution without breaking clients
- Offset enables efficient seeking into result set

### Source Aggregation

The endpoint reads from:
1. `friction/*.yaml` — friction log entries
2. Questions extracted from spec artifacts
3. `specs/tasks.yaml` — task definitions

All sources are loaded at runtime from the spec root, ensuring the endpoint reflects the current governance state.

## Test Coverage

- **BDD scenarios**: `specs/features/platform_issues.feature` tagged `@AC-GOV-025`
  - Happy path: list issues with default ordering
  - Filtering: by kind, status, text search
  - Pagination: cursor navigation, limit handling
  - Error cases: invalid cursor, mixed params, oversized cursor
- **Integration tests**: `crates/app-http/tests/issues_api.rs`
  - `test_issues_endpoint_pagination`

## Known Limits

- **No real-time updates** — endpoint reflects file system state at request time
- **Cursor invalidation** — if underlying data changes significantly, old cursors may skip or duplicate items
- **Performance** — for very large issue sets (>1000), consider caching or materialized views

## Future Work

- Add `/platform/issues/{id}` for single-issue detail view
- Support GraphQL-style field selection
- Add webhook/SSE for real-time issue updates
- Integrate with GitHub Issues for bidirectional sync
