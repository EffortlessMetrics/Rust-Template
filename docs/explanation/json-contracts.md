---
id: EXPLANATION-TPL-JSON-CONTRACTS-001
title: JSON Contracts for CLI and Platform APIs
doc_type: explanation
status: published
audience: idp-implementers, agents, platform-engineers
tags: [json, api, contracts, idp, agents]
stories: [US-TPL-PLT-001]
requirements:
  - REQ-TPL-AI-IDP-COMPAT
  - REQ-TPL-PLATFORM-INTROSPECTION
  - REQ-TPL-PLATFORM-SCHEMA
acs:
  - AC-TPL-CLI-JSON-CORE
  - AC-TPL-CLI-JSON-OUTPUT
  - AC-TPL-AGENT-HINTS-SCHEMA
  - AC-TPL-PLATFORM-SCHEMA
  - AC-TPL-PLATFORM-GRAPH
  - AC-TPL-PLATFORM-DEVEX
adrs: [ADR-0001, ADR-0005]
last_updated: 2025-12-22
---

# JSON Contracts for CLI and Platform APIs

**Template Version:** v3.3.12

This document defines the JSON contracts for all machine-readable outputs from the Rust-as-Spec platform cell. These contracts enable AI agents, IDPs (Internal Developer Portals), and automation tools to integrate with the platform reliably.

## Why JSON Contracts Matter

The platform exposes structured data through two primary surfaces:

1. **CLI outputs** (`cargo xtask ... --json`) - for scripts and CI pipelines
2. **HTTP endpoints** (`/platform/*`) - for runtime introspection

Both surfaces emit JSON that agents and integrations depend on. This document serves as the **stability contract** for these outputs.

## Stability Guarantees

**Additive-only changes across patch releases:**
- New fields may be added to JSON objects
- New endpoints may be added
- Existing fields will not be removed or renamed without a major version bump
- Field types will not change without a major version bump

**Breaking changes require:**
- Major version bump (X.0.0)
- Documented migration path in CHANGELOG
- At least one minor release with deprecation warnings

---

## CLI JSON Outputs

### `cargo xtask ac-status --json`

Returns AC (Acceptance Criteria) health status.

**Top-level shape:**
```json
{
  "template_version": "3.3.8",
  "timestamp": "2025-12-01T10:00:00Z",
  "summary": {
    "total": 85,
    "passing": 80,
    "failing": 3,
    "unknown": 2
  },
  "stories": [
    {
      "id": "US-TPL-001",
      "title": "Story title",
      "requirements": [
        {
          "id": "REQ-TPL-HEALTH",
          "title": "Requirement title",
          "acs": [
            {
              "id": "AC-TPL-001",
              "text": "AC description",
              "status": "passing",
              "scenarios": ["Scenario name"]
            }
          ]
        }
      ]
    }
  ]
}
```

**Fields:**
- `template_version` (string): Current template version
- `timestamp` (string): ISO 8601 timestamp
- `summary.total` (int): Total AC count
- `summary.passing` (int): ACs with all scenarios passing
- `summary.failing` (int): ACs with any failing scenario
- `summary.unknown` (int): ACs without test coverage
- `stories[]`: Array of user stories
- `stories[].requirements[]`: Nested requirements
- `stories[].requirements[].acs[]`: Nested acceptance criteria
- `acs[].status` (string): One of `"passing"`, `"failing"`, `"unknown"`

**Governing ACs:** AC-TPL-CLI-JSON-CORE, AC-TPL-CLI-JSON-OUTPUT

---

### `cargo xtask version --json`

Returns version and build information.

**Top-level shape:**
```json
{
  "version": "3.3.8",
  "git_sha": "abc123def",
  "build_date": "2025-12-01"
}
```

**Fields:**
- `version` (string): Semantic version
- `git_sha` (string): Git commit SHA (or "unknown")
- `build_date` (string): Build date (or "unknown")

---

### `cargo xtask friction-list --json`

Returns DevEx friction log entries.

**Top-level shape:**
```json
{
  "entries": [
    {
      "id": "FRICTION-ENV-001",
      "date": "2025-11-30",
      "severity": "medium",
      "category": "environment",
      "summary": "Brief description",
      "status": "open"
    }
  ],
  "summary": {
    "total": 5,
    "open": 3,
    "by_severity": {
      "low": 1,
      "medium": 2,
      "high": 1,
      "critical": 0
    }
  }
}
```

**Fields:**
- `entries[]`: Array of friction entries
- `entries[].id` (string): Unique friction ID
- `entries[].severity` (string): One of `"low"`, `"medium"`, `"high"`, `"critical"`
- `entries[].status` (string): One of `"open"`, `"resolved"`, `"wont_fix"`

---

### `cargo xtask questions-list --json`

Returns design questions and ambiguities.

**Top-level shape:**
```json
{
  "questions": [
    {
      "id": "Q-001",
      "summary": "Question summary",
      "status": "open",
      "context": {
        "flow": "feature-dev"
      }
    }
  ],
  "counts": {
    "open": 2,
    "answered": 1,
    "resolved": 3,
    "total": 6
  }
}
```

---

## HTTP Platform APIs

### `GET /platform/status`

Returns overall platform health and governance state.

**Top-level shape:**
```json
{
  "service": {
    "service_id": "rust-template",
    "template_version": "3.3.8",
    "display_name": "My Service",
    "description": "Service description",
    "links": {},
    "tags": []
  },
  "governance": {
    "ledger": {
      "stories": 3,
      "requirements": 45,
      "acs": 85
    },
    "devex": {
      "commands": 25,
      "flows": 8
    },
    "docs": {
      "total": 75,
      "design": 30,
      "doc_type_issues": 0
    },
    "tasks": {
      "total": 12
    },
    "questions": {
      "open": 2,
      "answered": 1,
      "resolved": 3,
      "total": 6
    },
    "friction": {
      "total": 5,
      "open": 3,
      "by_severity": { "low": 1, "medium": 2, "high": 1, "critical": 0 },
      "recent": []
    },
    "forks": {
      "total": 2,
      "ids": ["my-service-v1", "my-service-v2"]
    },
    "policies": {
      "status": "passing"
    }
  },
  "config": {
    "env": "dev",
    "http_port": 8080,
    "settings": {},
    "secrets_redacted": {},
    "auth": {
      "mode": "basic",
      "token_present": true
    }
  }
}
```

**Key fields:**
- `governance.docs.doc_type_issues` (int): Count of docs failing doc_type contracts
- `governance.policies.status` (string): Policy check result
- `config.secrets_redacted` (object): Secret keys with "[REDACTED]" values

**Governing ACs:** AC-TPL-PLATFORM-SCHEMA, AC-TPL-LOG-NO-SECRETS

---

### `GET /platform/graph`

Returns the full governance graph (stories -> requirements -> ACs -> tests -> docs).

**Top-level shape:**
```json
{
  "nodes": [
    {
      "id": "US-TPL-001",
      "type": "story",
      "label": "Story title"
    },
    {
      "id": "REQ-TPL-HEALTH",
      "type": "requirement",
      "label": "Requirement title"
    }
  ],
  "edges": [
    {
      "from": "US-TPL-001",
      "to": "REQ-TPL-HEALTH",
      "relation": "has_requirement"
    }
  ]
}
```

**Node types:** `"story"`, `"requirement"`, `"ac"`, `"test"`, `"doc"`, `"command"`

**Edge relations:** `"has_requirement"`, `"has_ac"`, `"tested_by"`, `"documented_by"`, `"invokes"`

**Governing ACs:** AC-TPL-PLATFORM-GRAPH

---

### `GET /platform/docs/index`

Returns documentation index with health information.

**Top-level shape:**
```json
{
  "schema_version": "1.0",
  "template_version": "3.3.8",
  "docs": [
    {
      "id": "DESIGN-TPL-HEALTH-001",
      "file": "docs/design/health-endpoint.md",
      "doc_type": "design_doc",
      "stories": ["US-TPL-001"],
      "requirements": ["REQ-TPL-HEALTH"],
      "acs": ["AC-TPL-001"],
      "adrs": ["ADR-0001"],
      "doc_type_valid": true,
      "doc_type_issue": null
    }
  ],
  "summary": {
    "total": 75,
    "valid": 74,
    "with_issues": 1
  }
}
```

**Fields:**
- `docs[].doc_type_valid` (bool): Whether doc passes doc_type contract
- `docs[].doc_type_issue` (string|null): Issue description if invalid
- `summary.with_issues` (int): Count of docs with doc_type issues

**Governing ACs:** AC-TPL-PLATFORM-DOCS

---

### `GET /platform/agent/hints`

Returns prioritized work suggestions for AI agents.

**Top-level shape:**
```json
{
  "hints": [
    {
      "kind": "task",
      "priority": 1,
      "task_id": "TASK-001",
      "requirement_id": "REQ-TPL-FEATURE",
      "ac_ids": ["AC-TPL-XXX"],
      "summary": "Task description",
      "suggested_skill": "governed-feature-dev"
    }
  ],
  "total": 5
}
```

**Fields:**
- `hints[].kind` (string): Hint type (currently always `"task"`)
- `hints[].priority` (int): Lower = higher priority
- `hints[].suggested_skill` (string|null): Recommended skill to use

**Governing ACs:** AC-TPL-AGENT-HINTS-SCHEMA

---

### `GET /platform/tasks`

Returns tasks with optional filtering.

**Query parameters:**
- `status`: Filter by status (`Todo`, `InProgress`, `Review`, `Done`)
- `req`: Filter by requirement ID

**Top-level shape:**
```json
{
  "tasks": [
    {
      "id": "TASK-001",
      "title": "Task title",
      "requirement": "REQ-TPL-XXX",
      "acs": ["AC-TPL-XXX"],
      "status": "Todo",
      "owner": null,
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

### `GET /platform/devex/flows`

Returns developer experience flows and xtask commands.

**Top-level shape:**
```json
{
  "commands": [
    {
      "name": "check",
      "description": "Run fmt, clippy, and unit tests"
    }
  ],
  "flows": [
    {
      "name": "feature-dev",
      "description": "AC-first feature development",
      "steps": ["Step 1", "Step 2"]
    }
  ]
}
```

**Governing ACs:** AC-TPL-PLATFORM-DEVEX

---

## Using JSON Contracts

### For AI Agents

```bash
# Get prioritized work
curl http://localhost:8080/platform/agent/hints | jq '.hints[0]'

# Check governance health before making changes
curl http://localhost:8080/platform/status | jq '.governance'

# Find docs with issues
curl http://localhost:8080/platform/docs/index | jq '.docs[] | select(.doc_type_valid == false)'
```

### For CI Pipelines

```bash
# Fail if any ACs are failing
ac_status=$(cargo xtask ac-status --json)
failing=$(echo "$ac_status" | jq '.summary.failing')
if [ "$failing" -gt 0 ]; then
  echo "AC failures detected: $failing"
  exit 1
fi
```

### For IDPs

IDPs can poll `/platform/status` to build dashboards showing:
- AC health across services
- Documentation coverage
- Friction log trends
- Task progress

---

## Related Documentation

- [IDP Positioning](./idp-positioning.md) - How platform cells integrate with IDPs
- [Template Contracts](./TEMPLATE-CONTRACTS.md) - Kernel requirements and extension points
- [Platform Introspection Design](../design/platform-introspection.md) - Technical design
