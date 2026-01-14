---
id: API-FRICTION-001
title: Platform Friction API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, friction, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Friction API

The Friction API provides endpoints for tracking development friction, including process problems, tooling pain points, and DevEx issues discovered during development workflows.

## Endpoints

### GET /platform/friction

Returns all development friction entries.

#### Request Example

```bash
curl http://localhost:8080/platform/friction
```

#### Response

```json
{
  "entries": [
    {
      "id": "FRICTION-EXAMPLE-001",
      "date": "2025-11-26",
      "category": "tooling",
      "severity": "medium",
      "summary": "Slow compile times in debug mode",
      "description": "Debug builds take 5+ minutes, slowing development iteration",
      "expected_behavior": "Debug builds should complete in under 2 minutes",
      "workaround": "Use release builds for testing when possible",
      "impact": "Slows down development cycle",
      "context": {
        "discovered_by": "alice",
        "flow": "first-hour",
        "phase": "setup",
        "files_involved": ["Cargo.toml"],
        "commands_involved": ["cargo build"]
      },
      "status": "open",
      "resolution": null,
      "refs": ["REQ-EXAMPLE-001"],
      "related_items": {
        "issues": ["ISSUE-001"],
        "adrs": ["ADR-0019"],
        "tasks": ["TASK-001"]
      }
    }
  ],
  "total": 1
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `entries` | array | Yes | Array of friction entry objects |
| `entries[].id` | string | Yes | Friction identifier (e.g., "FRICTION-EXAMPLE-001") |
| `entries[].date` | string | Yes | ISO 8601 date when friction was discovered |
| `entries[].category` | string | Yes | Friction category (e.g., "tooling", "process", "ci_cd") |
| `entries[].severity` | string | Yes | Severity level |
| `entries[].summary` | string | Yes | One-line summary |
| `entries[].description` | string | Yes | Detailed description |
| `entries[].expected_behavior` | string | No | Expected behavior |
| `entries[].workaround` | string | No | Known workaround |
| `entries[].impact` | string | No | Impact assessment |
| `entries[].context` | object | No | Discovery context |
| `entries[].context.discovered_by` | string | No | Who discovered the friction |
| `entries[].context.flow` | string | No | Associated DevEx flow |
| `entries[].context.phase` | string | No | Flow phase |
| `entries[].context.files_involved` | array | Yes | Related file paths (may be empty) |
| `entries[].context.commands_involved` | array | Yes | Related commands (may be empty) |
| `entries[].status` | string | Yes | Friction status (default: "open") |
| `entries[].resolution` | object | No | Resolution details (if resolved) |
| `entries[].refs` | array | Yes | Reference IDs (may be empty) |
| `entries[].related_items` | object | No | Related governance items |
| `entries[].related_items.issues` | array | Yes | Related issue IDs (may be empty) |
| `entries[].related_items.adrs` | array | Yes | Related ADR IDs (may be empty) |
| `entries[].related_items.tasks` | array | Yes | Related task IDs (may be empty) |
| `total` | integer | Yes | Total number of friction entries |

---

### GET /platform/friction/{id}

Returns a specific friction entry by ID.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `id` | string | Yes | Friction ID (e.g., "FRICTION-EXAMPLE-001") |

#### Request Example

```bash
curl http://localhost:8080/platform/friction/FRICTION-EXAMPLE-001
```

#### Response

Same schema as `/platform/friction` but with a single entry object.

#### Error Responses

| Status | Description |
|---------|-------------|
| `404` | Friction entry not found |

---

## Friction Severity Values

| Severity | Priority | Description |
|----------|-----------|-------------|
| `critical` | 1 | Blocks development, requires immediate attention |
| `high` | 2 | Significantly impacts development |
| `medium` | 3 | Moderate impact on development |
| `low` | 4 | Minor inconvenience |

---

## Friction Status Values

| Status | Description |
|--------|-------------|
| `open` | Friction is unresolved |
| `investigating` | Under investigation |
| `resolved` | Friction has been resolved |
| `wont_fix` | Friction will not be fixed (documented as known limitation) |

---

## Friction Categories

Common friction categories include:

| Category | Description |
|----------|-------------|
| `tooling` | Issues with development tools or tooling setup |
| `process` | Workflow or process issues |
| `ci_cd` | CI/CD pipeline problems |
| `documentation` | Documentation gaps or inaccuracies |
| `performance` | Performance issues |
| `testing` | Testing framework or test infrastructure issues |

---

## Resolution Schema

When friction is resolved, the `resolution` field contains:

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `resolved_by` | string | Yes | Resolver identifier |
| `resolved_at` | string | Yes | ISO 8601 timestamp of resolution |
| `fix_description` | string | No | Description of the fix |
| `pr_links` | array | Yes | Related PR URLs (may be empty) |
| `verification` | string | No | Verification steps or notes |

---

## File Naming Convention

Friction files are stored in the `friction/` directory with the following naming pattern:

```
friction/
├── README.yaml
└── FRICTION-{DOMAIN}-{NUMBER}.yaml
```

Example: `FRICTION-TOOLING-001.yaml`

---

## Related Endpoints

- [`/platform/issues`](./issues.md) - Unified issues including friction
- [`/platform/status`](./status.md) - Friction counts and severity breakdown in platform status
- [`/platform/agent/hints`](./agent-hints.md) - Friction hints for agents

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Friction Directory](../../friction/)
