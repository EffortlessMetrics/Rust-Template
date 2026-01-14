---
id: API-AGENT-HINTS-001
title: Platform Agent Hints API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, agent-hints, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-AGENT-HINTS-SCHEMA]
acs: [AC-PLT-015, AC-TPL-AGENT-HINTS-SCHEMA]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Agent Hints API

The Agent Hints API provides prioritized work suggestions for AI agents based on task status, AC coverage, and kernel health. Includes both task hints and governance hints for failing kernel ACs.

## Endpoints

### GET /platform/agent/hints

Returns prioritized work suggestions for AI agents with optional filtering.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `owner` | string | No | Filter by task owner |
| `label` | string | No | Filter by task label |
| `requirement` | string | No | Filter by requirement ID |
| `kind` | string | No | Filter by hint kind ("task" or "governance") |

#### Request Examples

```bash
# Get all hints
curl http://localhost:8080/platform/agent/hints

# Get hints for a specific owner
curl "http://localhost:8080/platform/agent/hints?owner=alice"

# Get only governance hints (failing kernel ACs)
curl "http://localhost:8080/platform/agent/hints?kind=governance"

# Get hints for high-priority tasks
curl "http://localhost:8080/platform/agent/hints?label=priority:high"

# Get hints for a specific requirement
curl "http://localhost:8080/platform/agent/hints?requirement=REQ-MYSERV-001"
```

#### Response

```json
{
  "hints": [
    {
      "id": "hint-001",
      "kind": "task",
      "priority": "high",
      "status": "open",
      "reason": {
        "summary": "Implement acceptance criteria",
        "code": "TASK_PENDING",
        "details": "Task is pending implementation",
        "category": "task"
      },
      "target": {
        "Task": {
          "id": "implement_ac"
        }
      },
      "tags": ["priority:high", "backend"],
      "links": {
        "requirements": ["REQ-MYSERV-001"],
        "acs": ["AC-MYSERV-001"],
        "docs": [],
        "flows": []
      },
      "task_id": "implement_ac",
      "title": "Implement Acceptance Criteria",
      "owner": "alice",
      "labels": ["priority:high"],
      "requirement_ids": ["REQ-MYSERV-001"],
      "ac_ids": ["AC-MYSERV-001"],
      "recommended_sequence": [
        {
          "kind": "command",
          "value": "cargo xtask ac-new"
        },
        {
          "kind": "command",
          "value": "cargo xtask bdd"
        }
      ]
    },
    {
      "id": "hint-002",
      "kind": "governance",
      "priority": "high",
      "status": "open",
      "reason": {
        "summary": "Fix failing kernel AC",
        "code": "KERNEL_AC_FAILING",
        "details": "AC-PLT-001 is failing in BDD tests",
        "category": "governance"
      },
      "target": {
        "Ac": {
          "id": "AC-PLT-001"
        }
      },
      "tags": ["kernel", "priority:high"],
      "links": {
        "requirements": ["REQ-PLT-001"],
        "acs": ["AC-PLT-001"],
        "docs": [],
        "flows": []
      },
      "task_id": "AC-PLT-001",
      "title": "Fix failing kernel AC",
      "owner": "kernel",
      "labels": ["kernel", "priority:high"],
      "requirement_ids": ["REQ-PLT-001"],
      "ac_ids": ["AC-PLT-001"],
      "recommended_sequence": []
    }
  ],
  "warnings": [],
  "warnings_structured": []
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `hints` | array | Yes | Array of hint objects |
| `hints[].id` | string | Yes | Unique hint identifier |
| `hints[].kind` | string | Yes | Hint kind ("task", "governance", "policy", "flow") |
| `hints[].priority` | string | Yes | Priority level ("low", "medium", "high") |
| `hints[].status` | string | Yes | Hint status ("open", "in_progress", "done") |
| `hints[].reason` | object | Yes | Reason for this hint |
| `hints[].reason.summary` | string | Yes | One-line summary |
| `hints[].reason.code` | string | Yes | Reason code |
| `hints[].reason.details` | string | Yes | Detailed explanation |
| `hints[].reason.category` | string | No | Reason category |
| `hints[].target` | object | Yes | Target of this hint |
| `hints[].target.Task` | object | No | Task target (when kind="task") |
| `hints[].target.Task.id` | string | Yes | Task ID |
| `hints[].target.Ac` | object | No | AC target (when kind="governance") |
| `hints[].target.Ac.id` | string | Yes | AC ID |
| `hints[].tags` | array | Yes | Associated tags (may be empty `[]`) |
| `hints[].links` | object | Yes | Related links |
| `hints[].links.requirements` | array | Yes | Related requirement IDs (may be empty) |
| `hints[].links.acs` | array | Yes | Related AC IDs (may be empty) |
| `hints[].links.docs` | array | Yes | Related documentation paths (may be empty) |
| `hints[].links.flows` | array | Yes | Related flow names (may be empty) |
| `hints[].task_id` | string | Yes | Task ID (convenience field) |
| `hints[].title` | string | Yes | Task title (convenience field) |
| `hints[].owner` | string | Yes | Task owner (convenience field) |
| `hints[].labels` | array | Yes | Task labels (convenience field) |
| `hints[].requirement_ids` | array | Yes | Related requirement IDs (convenience field) |
| `hints[].ac_ids` | array | Yes | Related AC IDs (convenience field) |
| `hints[].recommended_sequence` | array | Yes | Recommended command sequence (may be empty `[]`) |
| `hints[].recommended_sequence[].kind` | string | Yes | Step kind ("command") |
| `hints[].recommended_sequence[].value` | string | Yes | Command value |
| `warnings` | array | Yes | Human-readable warning messages (may be empty `[]`) |
| `warnings_structured` | array | Yes | Structured referential integrity warnings (may be empty `[]`) |

---

## Hint Kinds

| Kind | Description | Target Type |
|-------|-------------|-------------|
| `task` | Work suggestions for tasks | `Task` |
| `governance` | Kernel governance issues (failing ACs) | `Ac` |
| `policy` | Policy-related hints | N/A |
| `flow` | Flow-related hints | N/A |

---

## Priority Levels

| Priority | Description |
|----------|-------------|
| `low` | Low priority work |
| `medium` | Medium priority work |
| `high` | High priority work |

---

## Hint Status Values

| Status | Description |
|--------|-------------|
| `open` | Hint is pending |
| `in_progress` | Hint is being worked on |
| `done` | Hint is completed |

---

## Referential Integrity Warnings

The `warnings_structured` array contains structured warnings about invalid references:

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `message` | string | Yes | Human-readable warning message |
| `invalid_refs` | array | Yes | Invalid reference IDs |
| `ref_type` | string | Yes | Reference type ("AC" or "REQ") |

Example warning:

```json
{
  "message": "Invalid AC reference AC-INVALID-001 in task implement_feature",
  "invalid_refs": ["AC-INVALID-001"],
  "ref_type": "AC"
}
```

---

## Sorting

Hints are sorted by:

1. **Status** (in_progress before open)
2. **Priority label** (high > medium > low > none)
3. **Task ID** (alphabetical for tiebreaking)

### Priority Label Order

| Label | Order |
|-------|--------|
| `priority:high` or `high` | 0 (highest) |
| `priority:medium` or `medium` | 1 |
| `priority:low` or `low` | 2 |
| None | 3 (lowest) |

---

## Filtering

### Owner Filter

Case-insensitive match against task owner field.

### Label Filter

Case-insensitive match against task labels.

### Requirement Filter

Case-insensitive match against requirement IDs in `requirement_ids` array.

### Kind Filter

Exact match against `kind` field ("task" or "governance").

---

## Recommended Command Sequences

The `recommended_sequence` array provides a sequence of commands to complete the task. Each step is:

| Field | Description |
|-------|-------------|
| `kind` | Always "command" |
| `value` | The command to execute |

Common commands include:
- `cargo xtask bundle <task_id>` - Create work bundle
- `cargo xtask test-ac <ac_id>` - Test specific AC
- `cargo xtask bdd` - Run BDD tests
- `cargo xtask selftest` - Run kernel selftest
- `cargo xtask ac-new` - Create new AC
- `cargo xtask adr-new` - Create new ADR

---

## Kernel Governance Hints

When `kind="governance"`, hints represent failing kernel ACs that should be fixed before other work:

- **Priority**: Always "high"
- **Owner**: Always "kernel"
- **Target**: Always an AC (`Ac` target type)
- **Recommended Sequence**: Always empty (kernel ACs don't have flow sequences)

These hints help agents prioritize fixing failing kernel ACs before working on other tasks.

---

## Related Endpoints

- [`/platform/tasks`](./tasks.md) - Task management endpoints
- [`/platform/coverage`](./overview/index.md) - AC coverage data
- [`/platform/status`](./status.md) - Platform status with governance health

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Agent Guide](../AGENT_GUIDE.md)
- [IDP Cell Contract](../IDP_CELL_CONTRACT.md)
