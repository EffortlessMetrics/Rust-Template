---
id: API-STATUS-001
title: Platform Status API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, status, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS]
acs: [AC-PLT-015, AC-TPL-ERROR-MAPPING]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

<!-- doclint:disable orphan-version -->
<!-- Note: JSON examples contain version strings that are intentionally not tied to template version -->

# Platform Status API

The Status API provides comprehensive platform health, governance metrics, AC coverage, and runtime configuration information.

## Endpoints

### GET /platform/status

Returns comprehensive platform health, governance metrics, AC coverage, and runtime configuration.

#### Request Example

```bash
curl http://localhost:8080/platform/status
```

#### Response

<!-- doclint:disable orphan-version -->
```json
{
  "service": {
    "service_id": "my-service",
    "template_version": "3.3.9",
    "display_name": "My Service",
    "description": "Example service description",
    "links": {
      "kernel_contract": "https://example.com/kernel-contract",
      "roadmap": "https://example.com/roadmap",
      "agent_guide": "https://example.com/agent-guide",
      "feature_status": "https://example.com/feature-status",
      "support": "https://example.com/support"
    },
    "tags": ["rust", "api", "service"]
  },
  "governance": {
    "ledger": {
      "stories": 5,
      "requirements": 12,
      "acs": 25
    },
    "devex": {
      "commands": 15,
      "flows": 8
    },
    "docs": {
      "total": 42,
      "design": 8,
      "doc_type_issues": 3
    },
    "tasks": {
      "total": 10,
      "by_status": {
        "todo": 3,
        "in_progress": 4,
        "review": 2,
        "done": 1
      }
    },
    "questions": {
      "open": 2,
      "answered": 3,
      "resolved": 5,
      "total": 10,
      "top_open": [
        {
          "id": "Q-EXAMPLE-001",
          "summary": "Should we use async/await?",
          "flow": "bundle"
        },
        {
          "id": "Q-EXAMPLE-002",
          "summary": "Which database to use?",
          "flow": "first-hour"
        }
      ]
    },
    "friction": {
      "total": 8,
      "open": 3,
      "by_severity": {
        "low": 2,
        "medium": 4,
        "high": 1,
        "critical": 1
      },
      "recent": [
        {
          "id": "FRICTION-EXAMPLE-001",
          "date": "2025-11-26",
          "severity": "high",
          "summary": "Slow compile times",
          "category": "tooling"
        }
      ]
    },
    "forks": {
      "total": 2,
      "ids": ["FORK-EXAMPLE-001", "FORK-EXAMPLE-002"]
    },
    "policies": {
      "status": "pass"
    },
    "ac_coverage": {
      "total": 25,
      "passing": 20,
      "failing": 3,
      "unknown": 2
    }
  },
  "config": {
    "env": "dev",
    "http_port": 8080,
    "settings": {
      "platform.auth_mode": "basic",
      "platform.log_level": "info"
    },
    "secrets_redacted": {
      "platform.auth_token": "[REDACTED]",
      "database.url": "[REDACTED]"
    },
    "auth": {
      "mode": "basic",
      "token_present": true
    }
  },
  "errors": {
    "has_recent_errors": false,
    "last_error": null,
    "stats": {
      "total_errors": 0,
      "client_errors": 0,
      "server_errors": 0
    }
  }
}
```
<!-- doclint:enable orphan-version -->

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `service` | object | Yes | Service metadata |
| `service.service_id` | string | Yes | Unique service identifier |
| `service.template_version` | string | Yes | Template version |
| `service.display_name` | string | No | Human-readable service name |
| `service.description` | string | No | Service description |
| `service.links` | object | Yes | Related links (may be empty `{}`) |
| `service.tags` | array | Yes | Service tags (may be empty `[]`) |
| `governance` | object | Yes | Governance metrics |
| `governance.ledger` | object | Yes | Spec ledger counts |
| `governance.ledger.stories` | integer | Yes | Number of stories |
| `governance.ledger.requirements` | integer | Yes | Number of requirements |
| `governance.ledger.acs` | integer | Yes | Number of acceptance criteria |
| `governance.devex` | object | Yes | DevEx metrics |
| `governance.devex.commands` | integer | Yes | Number of DevEx commands |
| `governance.devex.flows` | integer | Yes | Number of DevEx flows |
| `governance.docs` | object | Yes | Documentation metrics |
| `governance.docs.total` | integer | Yes | Total number of documents |
| `governance.docs.design` | integer | Yes | Number of design documents |
| `governance.docs.doc_type_issues` | integer | Yes | Number of docs with type contract issues |
| `governance.tasks` | object | Yes | Task metrics |
| `governance.tasks.total` | integer | Yes | Total number of tasks |
| `governance.tasks.by_status` | object | No | Task status breakdown |
| `governance.tasks.by_status.todo` | integer | Yes | Number of Todo tasks |
| `governance.tasks.by_status.in_progress` | integer | Yes | Number of InProgress tasks |
| `governance.tasks.by_status.review` | integer | Yes | Number of Review tasks |
| `governance.tasks.by_status.done` | integer | Yes | Number of Done tasks |
| `governance.questions` | object | Yes | Question metrics |
| `governance.questions.open` | integer | Yes | Number of open questions |
| `governance.questions.answered` | integer | Yes | Number of answered questions |
| `governance.questions.resolved` | integer | Yes | Number of resolved questions |
| `governance.questions.total` | integer | Yes | Total number of questions |
| `governance.questions.top_open` | array | Yes | Top 3 open questions (may be empty `[]`) |
| `governance.questions.top_open[].id` | string | Yes | Question ID |
| `governance.questions.top_open[].summary` | string | Yes | Question summary |
| `governance.questions.top_open[].flow` | string | Yes | Associated flow |
| `governance.friction` | object | Yes | Friction metrics |
| `governance.friction.total` | integer | Yes | Total number of friction entries |
| `governance.friction.open` | integer | Yes | Number of open friction entries |
| `governance.friction.by_severity` | object | Yes | Friction by severity |
| `governance.friction.by_severity.low` | integer | Yes | Number of low severity friction |
| `governance.friction.by_severity.medium` | integer | Yes | Number of medium severity friction |
| `governance.friction.by_severity.high` | integer | Yes | Number of high severity friction |
| `governance.friction.by_severity.critical` | integer | Yes | Number of critical severity friction |
| `governance.friction.recent` | array | Yes | Top 5 recent friction (may be empty `[]`) |
| `governance.friction.recent[].id` | string | Yes | Friction ID |
| `governance.friction.recent[].date` | string | Yes | ISO 8601 date |
| `governance.friction.recent[].severity` | string | Yes | Severity level |
| `governance.friction.recent[].summary` | string | Yes | One-line summary |
| `governance.friction.recent[].category` | string | Yes | Friction category |
| `governance.forks` | object | Yes | Fork metrics |
| `governance.forks.total` | integer | Yes | Total number of forks |
| `governance.forks.ids` | array | Yes | Fork IDs (may be empty `[]`) |
| `governance.policies` | object | Yes | Policy status |
| `governance.policies.status` | string | Yes | Policy enforcement status |
| `governance.ac_coverage` | object | No | AC coverage metrics |
| `governance.ac_coverage.total` | integer | Yes | Total number of ACs |
| `governance.ac_coverage.passing` | integer | Yes | Number of passing ACs |
| `governance.ac_coverage.failing` | integer | Yes | Number of failing ACs |
| `governance.ac_coverage.unknown` | integer | Yes | Number of unknown ACs |
| `config` | object | No | Runtime configuration (omitted if not loaded) |
| `config.env` | string | No | Environment mode |
| `config.http_port` | integer | Yes | HTTP port number |
| `config.settings` | object | Yes | Configuration settings (may be empty `{}`) |
| `config.secrets_redacted` | object | Yes | Redacted secrets (may be empty `{}`) |
| `config.auth` | object | Yes | Authentication summary |
| `config.auth.mode` | string | Yes | Auth mode label |
| `config.auth.token_present` | boolean | Yes | Whether auth token is configured |
| `errors` | object | Yes | Error tracking summary |
| `errors.has_recent_errors` | boolean | Yes | Whether any errors occurred since service start |
| `errors.last_error` | object | No | Most recent error (if any) |
| `errors.last_error.category` | string | Yes | Error category |
| `errors.last_error.message` | string | Yes | Human-readable error message |
| `errors.last_error.status_code` | integer | Yes | HTTP status code returned |
| `errors.last_error.occurred_at` | string | Yes | ISO 8601 timestamp |
| `errors.last_error.request_id` | string | No | X-Request-ID for correlation |
| `errors.stats` | object | Yes | Aggregated error statistics |
| `errors.stats.total_errors` | integer | Yes | Total errors since service start |
| `errors.stats.client_errors` | integer | Yes | Number of 4xx client errors |
| `errors.stats.server_errors` | integer | Yes | Number of 5xx server errors |

---

## Policy Status Values

| Status | Description |
|--------|-------------|
| `pass` | All policies pass |
| `fail` | One or more policies fail |
| `unknown` | Policy status not yet evaluated (policy-test hasn't run) |

---

## Error Categories

| Category | Description |
|----------|-------------|
| `task_not_found` | Task ID not found in tasks.yaml |
| `invalid_transition` | Invalid task status transition |
| `validation` | Request validation failed |
| `internal` | Internal server error |

---

## Optional Fields

Fields marked as optional in the schema may be:
- Completely absent from the JSON response
- Present with `null` value

Consumers MUST handle both cases. Common optional fields include:
- `config` (if config not loaded)
- `ac_coverage` (if no coverage data available)
- `last_error` (if no errors have occurred)

---

## Related Endpoints

- [`/platform/coverage`](./overview/index.md) - Detailed AC coverage
- [`/platform/docs/index`](./overview/index.md) - Documentation inventory
- [`/platform/graph`](./overview/index.md) - Governance graph
- [`/platform/debug/info`](./overview/index.md) - Debug version info

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [Platform API Contract](../reference/platform_api_contract.md)
