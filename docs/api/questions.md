---
id: API-QUESTIONS-001
title: Platform Questions API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, questions, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-QUESTIONS-LOGGED]
acs: [AC-PLT-015, AC-TPL-QUESTIONS-LOGGED]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

# Platform Questions API

The Questions API provides endpoints for tracking design decisions and ambiguities during development workflows. Questions capture flow decision points, options, recommendations, and their resolutions.

## Endpoints

### GET /platform/questions

Returns all design questions with optional status filtering.

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `status` | string | No | Filter by question status ("open", "answered", "resolved") |

#### Request Examples

```bash
# Get all questions
curl http://localhost:8080/platform/questions

# Get only open questions
curl "http://localhost:8080/platform/questions?status=open"

# Get only answered questions
curl "http://localhost:8080/platform/questions?status=answered"
```

#### Response

```json
{
  "questions": [
    {
      "id": "Q-EXAMPLE-001",
      "summary": "Should we use async/await or tokio spawn?",
      "status": "open",
      "flow": "bundle",
      "phase": "selection",
      "created_at": "2025-11-26T00:00:00Z"
    }
  ],
  "total": 1
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `questions` | array | Yes | Array of question summary objects |
| `questions[].id` | string | Yes | Question identifier (e.g., "Q-EXAMPLE-001") |
| `questions[].summary` | string | Yes | One-line question summary |
| `questions[].status` | string | Yes | Question status (open, answered, resolved) |
| `questions[].flow` | string | Yes | Associated DevEx flow |
| `questions[].phase` | string | Yes | Flow phase where question was raised |
| `questions[].created_at` | string | Yes | ISO 8601 timestamp of creation |
| `total` | integer | Yes | Total number of questions returned |

---

### GET /platform/questions/{id}

Returns a specific question entry by ID with full details including options and recommendations.

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|--------|-----------|-------------|
| `id` | string | Yes | Question ID (e.g., "Q-EXAMPLE-001") |

#### Request Example

```bash
curl http://localhost:8080/platform/questions/Q-EXAMPLE-001
```

#### Response

```json
{
  "id": "Q-EXAMPLE-001",
  "task_id": "implement_feature",
  "req_ids": ["REQ-EXAMPLE-001"],
  "ac_ids": ["AC-EXAMPLE-001"],
  "refs": ["ADR-0016"],
  "summary": "Should we use async/await or tokio spawn?",
  "context": {
    "flow": "bundle",
    "phase": "selection",
    "description": "Choosing async runtime for the new feature",
    "files_involved": ["src/feature.rs"]
  },
  "options": [
    {
      "label": "async/await",
      "description": "Use Rust's native async/await syntax",
      "risk": "May require restructuring existing code",
      "reversible": true
    },
    {
      "label": "tokio::spawn",
      "description": "Spawn tasks using tokio",
      "risk": "Higher complexity",
      "reversible": true
    }
  ],
  "recommendation": {
    "option_label": "async/await",
    "rationale": "Native syntax is more idiomatic and has better tooling support",
    "confidence": "high"
  },
  "created_by": "flow",
  "created_at": "2025-11-26T00:00:00Z",
  "status": "open",
  "resolution": null
}
```

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `id` | string | Yes | Question identifier |
| `task_id` | string | No | Associated task ID |
| `req_ids` | array | Yes | Related requirement IDs (may be empty) |
| `ac_ids` | array | Yes | Related AC IDs (may be empty) |
| `refs` | array | Yes | Reference IDs (may be empty) |
| `summary` | string | Yes | One-line question summary |
| `context` | object | Yes | Question context |
| `context.flow` | string | Yes | Associated DevEx flow |
| `context.phase` | string | Yes | Flow phase |
| `context.description` | string | No | Additional context description |
| `context.files_involved` | array | Yes | Related file paths (may be empty) |
| `options` | array | Yes | Available options (may be empty) |
| `options[].label` | string | Yes | Option label |
| `options[].description` | string | Yes | Option description |
| `options[].risk` | string | No | Risk assessment |
| `options[].reversible` | boolean | Yes | Whether this choice can be reversed |
| `recommendation` | object | No | Recommended option (if resolved) |
| `recommendation.option_label` | string | Yes | Chosen option label |
| `recommendation.rationale` | string | Yes | Reason for recommendation |
| `recommendation.confidence` | string | No | Confidence level |
| `created_by` | string | Yes | Creator identifier |
| `created_at` | string | Yes | ISO 8601 timestamp |
| `status` | string | Yes | Question status (default: "open") |
| `resolution` | object | No | Resolution details (if resolved) |
| `resolution.resolved_by` | string | Yes | Resolver identifier |
| `resolution.resolved_at` | string | Yes | ISO 8601 timestamp |
| `resolution.chosen_option` | string | Yes | Chosen option label |
| `resolution.notes` | string | No | Additional notes |

---

## Question Status Values

| Status | Description |
|--------|-------------|
| `open` | Question is open and awaiting decision |
| `answered` | Question has been answered but not yet implemented |
| `resolved` | Question has been resolved and implemented |
| `obsolete` | Question is no longer relevant |

---

## File Naming Convention

Question files are stored in the `questions/` directory with the following naming pattern:

```
questions/
├── README.yaml
└── Q-{DOMAIN}-{NUMBER}.yaml
```

Example: `Q-EXAMPLE-001.yaml`

---

## Related Endpoints

- [`/platform/issues`](./issues.md) - Unified issues including questions
- [`/platform/status`](./status.md) - Question counts in platform status
- [`/platform/agent/hints`](./agent-hints.md) - Question hints for agents

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Questions Directory](../../questions/)
