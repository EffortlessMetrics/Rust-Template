# Platform API Reference

<!-- doclint:disable orphan-version -->
<!-- Contains example API responses with placeholder versions (e.g., 0.1.0) -->

**Version:** 3.3.9
**Last Updated:** 2025-12-12

This document provides a comprehensive reference for all `/platform/*` API endpoints and related introspection endpoints in the Rust-as-Spec template.

---

## Table of Contents

- [Core Platform Endpoints](#core-platform-endpoints)
- [Governance & Discovery](#governance--discovery)
- [Work & Tasks](#work--tasks)
- [Metadata & Issues](#metadata--issues)
- [Agent Support](#agent-support)
- [UI Endpoints](#ui-endpoints)
- [Service Endpoints](#service-endpoints)

---

## Core Platform Endpoints

### GET /platform/status

**Description:** Returns comprehensive platform health, governance metrics, AC coverage, and runtime configuration.

**Response Format:** JSON object with service metadata, governance status, AC coverage, task counts, friction/question counts, and runtime config.

**Example:**
```bash
curl http://localhost:8080/platform/status
```

**Response Structure:**
```json
{
  "service": {
    "service_id": "string",
    "template_version": "string",
    "display_name": "string (optional)",
    "description": "string (optional)",
    "links": {},
    "tags": []
  },
  "governance": {
    "ledger": {
      "stories": 0,
      "requirements": 0,
      "acs": 0
    },
    "devex": {
      "commands": 0,
      "flows": 0
    },
    "docs": {
      "total": 0,
      "design": 0,
      "doc_type_issues": 0
    },
    "tasks": {
      "total": 0,
      "by_status": {
        "todo": 0,
        "in_progress": 0,
        "review": 0,
        "done": 0
      }
    },
    "questions": {
      "open": 0,
      "answered": 0,
      "resolved": 0,
      "total": 0,
      "top_open": []
    },
    "friction": {
      "total": 0,
      "open": 0,
      "by_severity": {
        "low": 0,
        "medium": 0,
        "high": 0,
        "critical": 0
      },
      "recent": []
    },
    "forks": {
      "total": 0,
      "ids": []
    },
    "policies": {
      "status": "string"
    },
    "ac_coverage": {
      "total": 0,
      "passing": 0,
      "failing": 0,
      "unknown": 0
    }
  },
  "config": {
    "env": "string (optional)",
    "http_port": 8080,
    "settings": {},
    "secrets_redacted": {},
    "auth": {
      "mode": "string",
      "token_present": false
    }
  },
  "errors": {}
}
```

---

## Governance & Discovery

### GET /platform/graph

**Description:** Returns the full governance graph showing relationships between stories, requirements, ACs, tests, documentation, and DevEx commands.

**Response Format:** JSON object containing nodes and edges of the governance graph.

**Example:**
```bash
curl http://localhost:8080/platform/graph
```

---

### GET /platform/schema

**Description:** Returns machine-readable OpenAPI/JSON schema for all platform data structures.

**Response Format:** JSON object with schema definitions.

**Example:**
```bash
curl http://localhost:8080/platform/schema
```

---

### GET /platform/schema/{name}

**Description:** Returns a specific schema by name (e.g., "task", "friction", "question").

**Parameters:**
- `name` (path): Schema name to retrieve

**Response Format:** JSON object with the requested schema definition.

**Example:**
```bash
curl http://localhost:8080/platform/schema/task
```

---

### GET /platform/devex/flows

**Description:** Returns all developer experience flows and their associated xtask commands.

**Response Format:** JSON object with flow definitions from `specs/devex_flows.yaml`.

**Example:**
```bash
curl http://localhost:8080/platform/devex/flows
```

---

### GET /platform/docs/index

**Description:** Returns documentation inventory with health validation against doc_type contracts.

**Response Format:** JSON array of all documents with metadata, traceability links (stories, REQs, ACs, ADRs), and validation status.

**Example:**
```bash
curl http://localhost:8080/platform/docs/index
```

**Response Structure:**
```json
{
  "schema_version": "string",
  "template_version": "string",
  "docs": [
    {
      "id": "string",
      "file": "string",
      "doc_type": "string",
      "stories": [],
      "requirements": [],
      "acs": [],
      "adrs": [],
      "doc_type_valid": true,
      "doc_type_issue": "string (optional)"
    }
  ],
  "summary": {
    "total": 0,
    "valid": 0,
    "with_issues": 0
  }
}
```

---

### GET /platform/coverage

**Description:** Returns AC coverage summary with BDD test results mapped to acceptance criteria.

**Response Format:** JSON object with summary counts and detailed AC-by-AC status.

**Example:**
```bash
curl http://localhost:8080/platform/coverage
```

**Response Structure:**
```json
{
  "summary": {
    "passing": 0,
    "failing": 0,
    "unknown": 0,
    "total": 0
  },
  "details": [
    {
      "id": "AC-XXX-001",
      "title": "string",
      "status": "passing|failing|unknown",
      "story": "US-XXX-001",
      "requirement": "REQ-XXX-001",
      "scenarios": []
    }
  ]
}
```

---

### GET /platform/debug/info

**Description:** Returns basic kernel and template version information for debugging.

**Response Format:** JSON object with version metadata.

**Example:**
```bash
curl http://localhost:8080/platform/debug/info
```

**Response Structure:**
```json
{
  "kernel_version": "string",
  "template_version": "string"
}
```

---

## Work & Tasks

### GET /platform/tasks

**Description:** Returns all tasks from `specs/tasks.yaml` with optional filtering.

**Query Parameters:**
- `status` (optional): Filter by status (e.g., "Todo", "InProgress", "Review", "Done")
- `req` (optional): Filter by requirement ID

**Response Format:** JSON object with array of tasks.

**Example:**
```bash
# Get all tasks
curl http://localhost:8080/platform/tasks

# Get only Todo tasks
curl http://localhost:8080/platform/tasks?status=Todo

# Get tasks for a specific requirement
curl http://localhost:8080/platform/tasks?req=REQ-MYSERV-001
```

**Response Structure:**
```json
{
  "tasks": [
    {
      "id": "string",
      "title": "string",
      "requirement": "string",
      "acs": [],
      "status": "string",
      "owner": "string (optional)",
      "labels": [],
      "docs": {
        "design": [],
        "plan": []
      }
    }
  ]
}
```

---

### GET /platform/tasks/suggest-next

**Description:** Returns recommended next task sequence based on a given task ID, considering dependencies and flows.

**Query Parameters:**
- `task` (required): Task ID to get suggestions for

**Response Format:** JSON object with suggested task sequence.

**Example:**
```bash
curl http://localhost:8080/platform/tasks/suggest-next?task=implement_ac
```

---

### GET /platform/tasks/graph

**Description:** Returns task dependency graph in JSON or Mermaid format.

**Query Parameters:**
- `format` (optional): Output format ("json" or "mermaid", defaults to "json")

**Response Format:** JSON object with task graph nodes and edges, or Mermaid diagram text.

**Example:**
```bash
# Get JSON graph
curl http://localhost:8080/platform/tasks/graph

# Get Mermaid diagram
curl http://localhost:8080/platform/tasks/graph?format=mermaid
```

---

### POST /platform/tasks/{id}/status

**Description:** Updates the status of a specific task.

**Parameters:**
- `id` (path): Task ID to update

**Request Body:**
```json
{
  "status": "Todo|InProgress|Review|Done"
}
```

**Response:** 204 No Content on success.

**Example:**
```bash
curl -X POST http://localhost:8080/platform/tasks/implement_ac/status \
  -H "Content-Type: application/json" \
  -d '{"status": "InProgress"}'
```

---

## Metadata & Issues

### GET /platform/friction

**Description:** Returns all development friction entries (DevEx issues, process problems, tooling pain points).

**Response Format:** JSON object with array of friction entries.

**Example:**
```bash
curl http://localhost:8080/platform/friction
```

**Response Structure:**
```json
{
  "entries": [
    {
      "id": "FRICTION-XXX-001",
      "date": "2025-12-10",
      "category": "string",
      "severity": "low|medium|high|critical",
      "summary": "string",
      "description": "string",
      "status": "open|resolved",
      "context": {},
      "resolution": {}
    }
  ],
  "total": 0
}
```

---

### GET /platform/friction/{id}

**Description:** Returns a specific friction entry by ID.

**Parameters:**
- `id` (path): Friction ID (e.g., "FRICTION-XXX-001")

**Response Format:** JSON object with friction entry details.

**Example:**
```bash
curl http://localhost:8080/platform/friction/FRICTION-XXX-001
```

---

### GET /platform/questions

**Description:** Returns all design questions with optional status filtering.

**Query Parameters:**
- `status` (optional): Filter by status ("open", "answered", "resolved")

**Response Format:** JSON object with array of question summaries.

**Example:**
```bash
# Get all questions
curl http://localhost:8080/platform/questions

# Get only open questions
curl http://localhost:8080/platform/questions?status=open
```

**Response Structure:**
```json
{
  "questions": [
    {
      "id": "Q-XXX-001",
      "summary": "string",
      "status": "open|answered|resolved",
      "flow": "string",
      "phase": "string",
      "created_at": "ISO 8601 timestamp"
    }
  ],
  "total": 0
}
```

---

### GET /platform/questions/{id}

**Description:** Returns a specific question entry by ID with full details including options and recommendations.

**Parameters:**
- `id` (path): Question ID (e.g., "Q-XXX-001")

**Response Format:** JSON object with complete question details.

**Example:**
```bash
curl http://localhost:8080/platform/questions/Q-XXX-001
```

---

### GET /platform/forks

**Description:** Returns all registered template forks/branches.

**Response Format:** JSON object with array of fork summaries.

**Example:**
```bash
curl http://localhost:8080/platform/forks
```

**Response Structure:**
```json
{
  "forks": [
    {
      "id": "FORK-XXX-001",
      "name": "string",
      "domain": "string",
      "status": "active|archived",
      "kernel_version": "string"
    }
  ],
  "total": 0
}
```

---

### GET /platform/forks/{name}

**Description:** Returns detailed information about a specific fork.

**Parameters:**
- `name` (path): Fork ID or name

**Response Format:** JSON object with complete fork details including features, pain points, and related items.

**Example:**
```bash
curl http://localhost:8080/platform/forks/FORK-XXX-001
```

---

### GET /platform/idp/snapshot

**Description:** Returns IDP (Internal Developer Platform) snapshot with governance health, AC coverage, and prioritized task hints for agents.

**Response Format:** JSON object with snapshot metadata and hints.

**Example:**
```bash
curl http://localhost:8080/platform/idp/snapshot
```

**Response Structure:**
```json
{
  "timestamp": "ISO 8601 timestamp",
  "template_version": "string",
  "service_id": "string (optional)",
  "governance_health": {
    "status": "healthy|degraded|failing",
    "ac_coverage": {
      "total": 0,
      "passing": 0,
      "failing": 0,
      "unknown": 0
    },
    "spec_counts": {
      "stories": 0,
      "requirements": 0,
      "acceptance_criteria": 0
    }
  },
  "documentation": {
    "total": 0,
    "valid": 0,
    "with_issues": 0
  },
  "task_hints": {
    "total_pending": 0,
    "total_in_progress": 0,
    "friction_count": 0,
    "question_count": 0,
    "high_priority": []
  }
}
```

---

## Agent Support

### GET /platform/agent/hints

**Description:** Returns prioritized work suggestions for AI agents based on task status, AC coverage, and kernel health. Includes both task hints and governance hints for failing kernel ACs.

**Query Parameters:**
- `owner` (optional): Filter by task owner
- `label` (optional): Filter by task label
- `requirement` (optional): Filter by requirement ID
- `kind` (optional): Filter by hint kind ("task" or "governance")

**Response Format:** JSON object with array of agent hints and referential integrity warnings.

**Example:**
```bash
# Get all hints
curl http://localhost:8080/platform/agent/hints

# Get hints for a specific owner
curl http://localhost:8080/platform/agent/hints?owner=alice

# Get only governance hints (failing kernel ACs)
curl http://localhost:8080/platform/agent/hints?kind=governance

# Get hints for high-priority tasks
curl http://localhost:8080/platform/agent/hints?label=priority:high
```

**Response Structure:**
```json
{
  "hints": [
    {
      "id": "hint-xxx",
      "kind": "task|governance|policy|flow",
      "priority": "low|medium|high",
      "status": "open|in_progress|done",
      "reason": {
        "summary": "string",
        "details": "string",
        "category": "string"
      },
      "target": {
        "Task": { "id": "string" }
      },
      "tags": [],
      "links": {
        "requirements": [],
        "acs": [],
        "docs": [],
        "flows": []
      },
      "task_id": "string",
      "title": "string",
      "owner": "string",
      "labels": [],
      "requirement_ids": [],
      "ac_ids": [],
      "recommended_sequence": [
        {
          "kind": "command",
          "value": "cargo xtask ..."
        }
      ]
    }
  ],
  "warnings": []
}
```

**Note:** Hints are sorted by:
1. Status (in_progress before open)
2. Priority (high > medium > low)
3. Task ID (alphabetical)

---

### GET /platform/ui/contract

**Description:** Returns the UI contract specification defining screens, regions, and stable `data-uiid` identifiers for tests and automation.

**Response Format:** JSON object with UI contract definition.

**Example:**
```bash
curl http://localhost:8080/platform/ui/contract
```

---

## UI Endpoints

These endpoints return HTML pages for human interaction with the platform.

### GET / or /ui

**Description:** Platform dashboard showing health metrics, AC coverage, and quick links.

**Example:**
```bash
curl http://localhost:8080/
```

---

### GET /ui/graph

**Description:** Visual governance graph showing relationships between stories, requirements, ACs, and docs (rendered with Mermaid).

**Example:**
```bash
curl http://localhost:8080/ui/graph
```

---

### GET /ui/flows

**Description:** Developer flows and tasks view with recommended sequences and task details.

**Example:**
```bash
curl http://localhost:8080/ui/flows
```

---

### GET /ui/coverage

**Description:** Interactive AC coverage table with filtering and search capabilities.

**Example:**
```bash
curl http://localhost:8080/ui/coverage
```

---

### GET /ui/tasks

**Description:** Kanban-style task board (Todo, In Progress, Review, Done) with HTMX-powered drag-and-drop.

**Example:**
```bash
curl http://localhost:8080/ui/tasks
```

---

## Service Endpoints

These are domain-specific endpoints for the example application logic.

### GET /health

**Description:** Service health check endpoint.

**Response Format:** JSON object with status.

**Example:**
```bash
curl http://localhost:8080/health
```

**Response:**
```json
{
  "status": "ok",
  "service": "service-api"
}
```

---

### GET /version

**Description:** Service version information.

**Response Format:** JSON object with version and git SHA.

**Example:**
```bash
curl http://localhost:8080/version
```

**Response:**
<!-- doclint:disable orphan-version -->
```json
{
  "version": "0.1.0",
  "gitSha": "abc123..."
}
```

---

### GET /metrics

**Description:** Prometheus-compatible metrics endpoint.

**Response Format:** Plain text metrics in Prometheus exposition format.

**Example:**
```bash
curl http://localhost:8080/metrics
```

---

### POST /api/echo

**Description:** Echo endpoint for testing error handling and request validation.

**Request Body:**
```json
{
  "message": "string"
}
```

**Response Format:** JSON object echoing the message.

**Example:**
```bash
curl -X POST http://localhost:8080/api/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

---

### GET /todos

**Description:** List all todos (example domain endpoint implementing AC-MYSERV-001 and AC-MYSERV-002).

**Response Format:** JSON array of todos.

**Example:**
```bash
curl http://localhost:8080/todos
```

---

### POST /todos

**Description:** Create a new todo (implements AC-MYSERV-003, AC-MYSERV-005, AC-MYSERV-006).

**Request Body:**
```json
{
  "id": "string",
  "title": "string (max 256 chars)"
}
```

**Response:** 201 Created with the created todo.

**Example:**
```bash
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"id": "todo-3", "title": "New task"}'
```

---

### DELETE /todos/{id}

**Description:** Delete a specific todo by ID (implements AC-MYSERV-004).

**Parameters:**
- `id` (path): Todo ID to delete

**Response:** 204 No Content on success, 404 if not found.

**Example:**
```bash
curl -X DELETE http://localhost:8080/todos/todo-1
```

---

### DELETE /todos/clear

**Description:** Clear all todos (for testing).

**Response:** 204 No Content.

**Example:**
```bash
curl -X DELETE http://localhost:8080/todos/clear
```

---

## Authentication

Most `/platform/*` endpoints respect the platform authentication configuration defined in `config/local.yaml` and `specs/config_schema.yaml`.

**Auth Modes:**
- `disabled`: No authentication required (development mode)
- `basic`: Requires Bearer token in Authorization header

**Example with token:**
```bash
curl http://localhost:8080/platform/status \
  -H "Authorization: Bearer your-token-here"
```

**Configuration:**
```yaml
# config/local.yaml
platform:
  auth_mode: basic
secrets:
  platform:
    auth_token: your-secret-token
```

---

## Related Documentation

- **Agent Guide:** `/home/steven/code/Rust/Rust-Template/docs/AGENT_GUIDE.md`
- **Missing Manual:** `/home/steven/code/Rust/Rust-Template/docs/MISSING_MANUAL.md`
- **How-to: Add HTTP Endpoint:** `/home/steven/code/Rust/Rust-Template/docs/how-to/add-http-endpoint.md`
- **Platform Contracts:** `/home/steven/code/Rust/Rust-Template/docs/explanation/TEMPLATE-CONTRACTS.md`

---

## Endpoint Summary Table

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/health` | Service health check | No |
| GET | `/version` | Service version info | No |
| GET | `/metrics` | Prometheus metrics | No |
| POST | `/api/echo` | Echo test endpoint | No |
| GET | `/platform/status` | Platform health & governance metrics | Yes* |
| GET | `/platform/graph` | Full governance graph | Yes* |
| GET | `/platform/schema` | All platform schemas | Yes* |
| GET | `/platform/schema/{name}` | Specific schema by name | Yes* |
| GET | `/platform/devex/flows` | DevEx flows & commands | Yes* |
| GET | `/platform/docs/index` | Documentation inventory | Yes* |
| GET | `/platform/coverage` | AC coverage summary | Yes* |
| GET | `/platform/debug/info` | Debug version info | Yes* |
| GET | `/platform/tasks` | List all tasks (with filters) | Yes* |
| GET | `/platform/tasks/suggest-next` | Suggested task sequence | Yes* |
| GET | `/platform/tasks/graph` | Task dependency graph | Yes* |
| POST | `/platform/tasks/{id}/status` | Update task status | Yes* |
| GET | `/platform/friction` | All friction entries | Yes* |
| GET | `/platform/friction/{id}` | Specific friction entry | Yes* |
| GET | `/platform/questions` | All questions (with filters) | Yes* |
| GET | `/platform/questions/{id}` | Specific question | Yes* |
| GET | `/platform/forks` | All template forks | Yes* |
| GET | `/platform/forks/{name}` | Specific fork details | Yes* |
| GET | `/platform/idp/snapshot` | IDP snapshot for agents | Yes* |
| GET | `/platform/agent/hints` | Prioritized work hints | Yes* |
| GET | `/platform/ui/contract` | UI contract specification | Yes* |
| GET | `/` or `/ui` | Platform dashboard (HTML) | No |
| GET | `/ui/graph` | Governance graph view (HTML) | No |
| GET | `/ui/flows` | Flows & tasks view (HTML) | No |
| GET | `/ui/coverage` | AC coverage view (HTML) | No |
| GET | `/ui/tasks` | Task board (HTML) | No |
| GET | `/todos` | List todos | No |
| POST | `/todos` | Create todo | No |
| DELETE | `/todos/{id}` | Delete todo | No |
| DELETE | `/todos/clear` | Clear all todos | No |

**\* Auth Required:** Depends on `platform.auth_mode` configuration. When set to `disabled`, no authentication is required.

---

## Common Use Cases

### For AI Agents

1. **Get prioritized work hints:**
   ```bash
   curl http://localhost:8080/platform/agent/hints
   ```

2. **Check governance health:**
   ```bash
   curl http://localhost:8080/platform/status
   ```

3. **Find failing ACs:**
   ```bash
   curl http://localhost:8080/platform/coverage
   ```

4. **Get tasks for a specific requirement:**
   ```bash
   curl http://localhost:8080/platform/tasks?req=REQ-MYSERV-001
   ```

### For CI/CD Pipelines

1. **Verify governance health:**
   ```bash
   curl http://localhost:8080/platform/status | jq '.governance.policies.status'
   ```

2. **Check AC coverage:**
   ```bash
   curl http://localhost:8080/platform/coverage | jq '.summary'
   ```

3. **Get IDP snapshot for deployment gates:**
   ```bash
   curl http://localhost:8080/platform/idp/snapshot
   ```

### For Developers

1. **Explore the governance graph:**
   ```bash
   curl http://localhost:8080/platform/graph | jq
   ```

2. **Find open friction issues:**
   ```bash
   curl http://localhost:8080/platform/friction | jq '.entries[] | select(.status == "open")'
   ```

3. **Get suggested next steps for a task:**
   ```bash
   curl http://localhost:8080/platform/tasks/suggest-next?task=implement_ac
   ```

---

**Note:** This reference is generated from the codebase at template version 3.3.9. For the most current endpoint definitions, refer to the source code in `crates/app-http/src/`.
