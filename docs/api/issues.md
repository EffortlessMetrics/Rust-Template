---
id: API-ISSUES-001
title: Platform Issues API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, issues, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Issues API

The Issues API provides a unified endpoint that normalizes friction, questions, and tasks into a common `Issue` representation with unified status, priority, and filtering capabilities.

## Endpoints

### GET /platform/issues

Returns all governance artifacts (friction, questions, tasks) as a unified list of issues with normalized status, priority, and filtering capabilities.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `kind` | string | No | Filter by issue kind ("friction", "question", "task") |
| `status` | string | No | Filter by normalized status ("open", "in_progress", "resolved") |
| `priority` | integer | No | Filter by exact priority (1=critical, 2=high, 3=medium, 4=low) |
| `min_priority` | integer | No | Filter by minimum priority (1=highest, inclusive) |
| `from_date` | string | No | Filter by date range start (ISO 8601) |
| `to_date` | string | No | Filter by date range end (ISO 8601) |
| `q` | string | No | Text search in id, summary, category, and labels |
| `page` | integer | No | Page number (1-indexed, default 1) |
| `per_page` | integer | No | Items per page (default 50, max 100) |

#### Request Examples

```bash
# Get all issues
curl http://localhost:8080/platform/issues

# Get open friction issues
curl "http://localhost:8080/platform/issues?kind=friction&status=open"

# Get high-priority issues (priority 1 or 2)
curl "http://localhost:8080/platform/issues?min_priority=2"

# Search issues
curl "http://localhost:8080/platform/issues?q=devex"

# Paginated request (page 2, 25 per page)
curl "http://localhost:8080/platform/issues?page=2&per_page=25"
```

#### Response

```json
{
  "issues": [
    {
      "id": "FRICTION-EXAMPLE-001",
      "kind": "friction",
      "status": "open",
      "native_status": "open",
      "summary": "Slow compile times in debug mode",
      "priority": 2,
      "created_at": "2025-11-26",
      "category": "tooling",
      "refs": ["REQ-EXAMPLE-001"],
      "owner": "alice",
      "labels": []
    },
    {
      "id": "Q-EXAMPLE-001",
      "kind": "question",
      "status": "open",
      "native_status": "open",
      "summary": "Should we use async/await or tokio spawn?",
      "priority": 3,
      "created_at": "2025-11-26T00:00:00Z",
      "category": "bundle",
      "refs": ["REQ-EXAMPLE-001", "AC-EXAMPLE-001"],
      "owner": "flow",
      "labels": []
    },
    {
      "id": "implement_feature",
      "kind": "task",
      "status": "in_progress",
      "native_status": "InProgress",
      "summary": "Implement new feature",
      "priority": 1,
      "created_at": null,
      "category": "REQ-EXAMPLE-001",
      "refs": ["REQ-EXAMPLE-001", "AC-EXAMPLE-001"],
      "owner": "alice",
      "labels": ["priority:high"]
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total_items": 10,
    "total_pages": 1
  },
  "summary": {
    "total": 10,
    "by_kind": {
      "friction": 3,
      "question": 2,
      "task": 5
    },
    "by_status": {
      "open": 6,
      "in_progress": 2,
      "resolved": 2
    }
  }
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `issues` | array | Yes | Array of issue objects |
| `issues[].id` | string | Yes | Unique identifier (FRICTION-XXX, Q-XXX, TASK-XXX) |
| `issues[].kind` | string | Yes | Issue type ("friction", "question", "task") |
| `issues[].status` | string | Yes | Normalized status ("open", "in_progress", "resolved") |
| `issues[].native_status` | string | Yes | Original/native status string (for transparency) |
| `issues[].summary` | string | Yes | One-line summary |
| `issues[].priority` | integer | Yes | Priority (1=critical/p0, 2=high/p1, 3=medium/p2, 4=low/p3) |
| `issues[].created_at` | string | No | Creation/discovery date (ISO 8601, nullable for tasks) |
| `issues[].category` | string | No | Category or flow context |
| `issues[].refs` | array | Yes | Related REQ/AC IDs (may be empty) |
| `issues[].owner` | string | No | Owner/assignee |
| `issues[].labels` | array | Yes | Labels/tags (may be empty) |
| `pagination` | object | Yes | Pagination metadata |
| `pagination.page` | integer | Yes | Current page number (1-indexed) |
| `pagination.per_page` | integer | Yes | Items per page |
| `pagination.total_items` | integer | Yes | Total number of items |
| `pagination.total_pages` | integer | Yes | Total number of pages |
| `summary` | object | Yes | Summary counts |
| `summary.total` | integer | Yes | Total number of issues |
| `summary.by_kind` | object | Yes | Counts by issue type |
| `summary.by_kind.friction` | integer | Yes | Friction count |
| `summary.by_kind.question` | integer | Yes | Question count |
| `summary.by_kind.task` | integer | Yes | Task count |
| `summary.by_status` | object | Yes | Counts by status |
| `summary.by_status.open` | integer | Yes | Open issues count |
| `summary.by_status.in_progress` | integer | Yes | In-progress issues count |
| `summary.by_status.resolved` | integer | Yes | Resolved issues count |

---

## Issue Kind Values

| Kind | Description | ID Pattern |
|-------|-------------|-------------|
| `friction` | Development friction entries | `FRICTION-{DOMAIN}-{NUMBER}` |
| `question` | Design questions | `Q-{DOMAIN}-{NUMBER}` |
| `task` | Task items | Task ID from tasks.yaml |

---

## Normalized Status Values

The API normalizes native statuses to canonical values:

| Native Status | Normalized Status | Kind |
|--------------|------------------|-------|
| `open` | `open` | friction, question |
| `investigating` | `in_progress` | friction |
| `in_progress` | `in_progress` | friction, question |
| `answered` | `in_progress` | question |
| `resolved` | `resolved` | friction, question |
| `wont_fix` | `resolved` | friction |
| `obsolete` | `resolved` | question |
| `Todo`, `open` | `open` | task |
| `InProgress` | `in_progress` | task |
| `Review` | `in_progress` | task |
| `Done`, `closed` | `resolved` | task |

---

## Priority Mapping

Priority is derived from native properties:

| Issue Kind | Priority Source | Values |
|-------------|------------------|---------|
| `friction` | Severity | critical=1, high=2, medium=3, low=4 |
| `question` | Default | Always 3 (medium) |
| `task` | Labels | `p0`=1, `p1`=2, `p2`=3, `p3`=4, default=3 |

---

## Sorting

Issues are sorted by:

1. **Priority** (highest first: 1 > 2 > 3 > 4)
2. **Date** (most recent first, for friction and questions)
3. **ID** (alphabetical, for tiebreaking)

---

## Filtering Behavior

### Text Search (`q` parameter)

Searches across:
- Issue ID (case-insensitive)
- Summary (case-insensitive)
- Category (case-insensitive)
- Labels (case-insensitive)

### Date Range

- `from_date`: Inclusive start date
- `to_date`: Inclusive end date
- Only applies to friction and questions (tasks don't have creation dates)

### Pagination

- `page`: 1-indexed (first page is 1)
- `per_page`: Default 50, maximum 100
- Empty result sets return `total_items: 0`

---

## Related Endpoints

- [`/platform/friction`](./friction.md) - Friction-specific endpoints
- [`/platform/questions`](./questions.md) - Question-specific endpoints
- [`/platform/tasks`](./tasks.md) - Task-specific endpoints
- [`/platform/status`](./status.md) - Issue counts in platform status

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
