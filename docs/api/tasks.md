---
id: API-TASKS-001
title: Platform Tasks API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, tasks, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-TASKS-HTTP]
acs: [AC-PLT-015, AC-TPL-TASKS-HTTP]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Tasks API

The Tasks API provides endpoints for managing development tasks, including listing, filtering, and updating task status.

## Endpoints

### GET /platform/tasks

Returns all tasks from `specs/tasks.yaml` with optional filtering. Task status is merged from the governance repository overlay.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `status` | string | No | Filter by task status (e.g., "Todo", "InProgress", "Review", "Done") |
| `req` | string | No | Filter by requirement ID |

#### Request Examples

```bash
# Get all tasks
curl http://localhost:8080/platform/tasks

# Get only Todo tasks
curl http://localhost:8080/platform/tasks?status=Todo

# Get tasks for a specific requirement
curl http://localhost:8080/platform/tasks?req=REQ-MYSERV-001
```

#### Response

```json
{
  "tasks": [
    {
      "id": "implement_ac",
      "title": "Implement Acceptance Criteria",
      "requirement": "REQ-MYSERV-001",
      "acs": ["AC-MYSERV-001", "AC-MYSERV-002"],
      "status": "Todo",
      "owner": "alice",
      "labels": ["priority:high"],
      "docs": {
        "design": ["docs/design/ac-implementation.md"],
        "plan": ["docs/implementation-plan.md"]
      }
    }
  ]
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `tasks` | array | Yes | Array of task objects |
| `tasks[].id` | string | Yes | Task identifier |
| `tasks[].title` | string | Yes | Task title |
| `tasks[].requirement` | string | Yes | Associated requirement ID |
| `tasks[].acs` | array | Yes | Associated acceptance criteria IDs |
| `tasks[].status` | string | Yes | Task status (Todo, InProgress, Review, Done) |
| `tasks[].owner` | string | No | Task owner/assignee |
| `tasks[].labels` | array | Yes | Task labels (may be empty) |
| `tasks[].docs` | object | No | Related documentation |
| `tasks[].docs.design` | array | No | Design documentation paths |
| `tasks[].docs.plan` | array | No | Implementation plan paths |

---

### GET /platform/tasks/suggest-next

Returns recommended next task sequence based on a given task ID, considering dependencies and flows.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `task` | string | Yes | Task ID to get suggestions for |

#### Request Example

```bash
curl "http://localhost:8080/platform/tasks/suggest-next?task=implement_ac"
```

#### Response

```json
{
  "task_id": "implement_ac",
  "suggested_sequence": [
    {
      "kind": "command",
      "value": "cargo xtask ac-new"
    },
    {
      "kind": "command",
      "value": "cargo xtask bdd"
    },
    {
      "kind": "command",
      "value": "cargo xtask test-ac AC-MYSERV-001"
    }
  ]
}
```

---

### GET /platform/tasks/graph

Returns task dependency graph in JSON or Mermaid format.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `format` | string | No | Output format: "json" or "mermaid" (default: "json") |

#### Request Examples

```bash
# Get JSON graph
curl http://localhost:8080/platform/tasks/graph

# Get Mermaid diagram
curl "http://localhost:8080/platform/tasks/graph?format=mermaid"
```

#### Response (JSON format)

```json
{
  "nodes": [
    {
      "id": "implement_ac",
      "title": "Implement Acceptance Criteria",
      "status": "Todo"
    },
    {
      "id": "write_tests",
      "title": "Write Tests",
      "status": "Todo"
    }
  ],
  "edges": [
    {
      "from": "implement_ac",
      "to": "write_tests",
      "label": "depends_on"
    }
  ]
}
```

#### Response (Mermaid format)

```json
{
  "mermaid": "graph TD\n  implement_ac[\"Implement AC\"]\n  write_tests[\"Write Tests\"]\n  implement_ac --> write_tests"
}
```

---

### POST /platform/tasks/{id}/status

Updates the status of a specific task.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `id` | string | Yes | Task ID to update |

#### Request Body

Supports both JSON and form-encoded formats.

**JSON format:**

```json
{
  "status": "InProgress"
}
```

**Form-encoded format:**

```
status=InProgress
```

#### Valid Status Values

| Value | Description |
|-------|-------------|
| `Todo` | Task not started |
| `InProgress` | Task in progress |
| `Review` | Task under review |
| `Done` | Task completed |

#### Request Examples

```bash
# Using JSON
curl -X POST http://localhost:8080/platform/tasks/implement_ac/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'

# Using form encoding
curl -X POST http://localhost:8080/platform/tasks/implement_ac/status \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d 'status=InProgress'
```

#### Response

- **Status Code:** `204 No Content`
- **Body:** Empty

#### Error Responses

| Status | Description |
|---------|-------------|
| `400` | Invalid status value or request format |
| `404` | Task not found |
| `422` | Invalid task status transition |

---

## Task Status Normalization

The API normalizes various status string formats to canonical values:

| Input Variants | Canonical Status |
|----------------|------------------|
| `open`, `Open`, `OPEN` | `Todo` |
| `inprogress`, `in_progress`, `in-progress`, `In_Progress` | `InProgress` |
| `review`, `Review` | `Review` |
| `done`, `closed`, `Closed` | `Done` |

Unknown statuses default to `Todo`.

---

## Related Endpoints

- [`/platform/agent/hints`](./agent-hints.md) - Prioritized task suggestions for agents
- [`/platform/issues`](./issues.md) - Unified issues including tasks
- [`/platform/status`](./status.md) - Task counts in platform status

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Task Lifecycle](../design/task-lifecycle.md)
