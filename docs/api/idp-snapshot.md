---
id: API-IDP-SNAPSHOT-001
title: Platform IDP Snapshot API
doc_type: reference
status: published
audience: developers, idp-operators, integration-developers
tags: [api, idp-snapshot, platform]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-IDP-SNAPSHOT]
acs: [AC-PLT-015]
adrs: [ADR-0005]
last_updated: 2025-12-27
---

<!-- doclint:disable orphan-version -->
<!-- Note: JSON examples contain version strings that are intentionally not tied to template version -->

# Platform IDP Snapshot API

The IDP Snapshot API provides a machine-readable contract for Internal Developer Platforms (IDPs) with governance health, AC coverage, and prioritized task hints for agents.

## Endpoints

### GET /platform/idp/snapshot

Returns IDP snapshot with governance health, AC coverage, documentation metrics, and task hints for agents.

#### Request Example

```bash
curl http://localhost:8080/platform/idp/snapshot
```

#### Response

<!-- doclint:disable orphan-version -->
```json
{
  "timestamp": "2025-12-27T14:30:00Z",
  "template_version": "3.3.9",
  "service_id": "my-service",
  "governance_health": {
    "status": "healthy",
    "ac_coverage": {
      "total": 25,
      "passing": 20,
      "failing": 3,
      "unknown": 2
    },
    "spec_counts": {
      "stories": 5,
      "requirements": 12,
      "acceptance_criteria": 25
    }
  },
  "documentation": {
    "total": 42,
    "valid": 39,
    "with_issues": 3
  },
  "task_hints": {
    "total_pending": 3,
    "total_in_progress": 4,
    "friction_count": 3,
    "question_count": 2,
    "high_priority": [
      {
        "task_id": "implement_feature",
        "title": "Implement New Feature",
        "status": "in_progress",
        "owner": "alice",
        "requirement_ids": ["REQ-MYSERV-001"],
        "ac_ids": ["AC-MYSERV-001", "AC-MYSERV-002"]
      },
      {
        "task_id": "fix_kernel_ac",
        "title": "Fix Failing Kernel AC",
        "status": "open",
        "owner": "bob",
        "requirement_ids": ["REQ-PLT-001"],
        "ac_ids": ["AC-PLT-001"]
      }
    ]
  }
}
```
<!-- doclint:enable orphan-version -->

#### Response Schema

| Field | Type | Required | Description |
|-------|------|-----------|-------------|
| `timestamp` | string | Yes | ISO 8601 timestamp of snapshot creation |
| `template_version` | string | Yes | Template version from spec_ledger.yaml |
| `service_id` | string | No | Service ID from service_metadata.yaml |
| `governance_health` | object | Yes | Governance health metrics |
| `governance_health.status` | string | Yes | Overall status ("healthy", "degraded", "failing") |
| `governance_health.ac_coverage` | object | Yes | AC coverage metrics |
| `governance_health.ac_coverage.total` | integer | Yes | Total number of ACs |
| `governance_health.ac_coverage.passing` | integer | Yes | Number of passing ACs |
| `governance_health.ac_coverage.failing` | integer | Yes | Number of failing ACs |
| `governance_health.ac_coverage.unknown` | integer | Yes | Number of unknown ACs |
| `governance_health.spec_counts` | object | Yes | Spec ledger counts |
| `governance_health.spec_counts.stories` | integer | Yes | Number of stories |
| `governance_health.spec_counts.requirements` | integer | Yes | Number of requirements |
| `governance_health.spec_counts.acceptance_criteria` | integer | Yes | Number of acceptance criteria |
| `documentation` | object | Yes | Documentation metrics |
| `documentation.total` | integer | Yes | Total number of documents |
| `documentation.valid` | integer | Yes | Number of valid documents |
| `documentation.with_issues` | integer | Yes | Number of documents with issues |
| `task_hints` | object | Yes | Task hints for agents |
| `task_hints.total_pending` | integer | Yes | Number of pending tasks |
| `task_hints.total_in_progress` | integer | Yes | Number of in-progress tasks |
| `task_hints.friction_count` | integer | Yes | Number of open friction entries |
| `task_hints.question_count` | integer | Yes | Number of open questions |
| `task_hints.high_priority` | array | Yes | Top 5 high-priority tasks (may be empty `[]`) |
| `task_hints.high_priority[].task_id` | string | Yes | Task ID |
| `task_hints.high_priority[].title` | string | Yes | Task title |
| `task_hints.high_priority[].status` | string | Yes | Task status |
| `task_hints.high_priority[].owner` | string | No | Task owner |
| `task_hints.high_priority[].requirement_ids` | array | Yes | Related requirement IDs |
| `task_hints.high_priority[].ac_ids` | array | Yes | Related AC IDs |

---

## Governance Health Status

| Status | Description | Criteria |
|--------|-------------|-----------|
| `healthy` | All systems operational | No failing ACs |
| `degraded` | Some issues present | One or more failing ACs |
| `failing` | Critical issues | Multiple or critical failing ACs |

---

## AC Coverage Sources

The `ac_coverage` metrics are derived from:

1. **Primary**: `target/ac_report.json` (Cucumber JSON from BDD tests)
2. **Fallback**: `docs/feature_status.md` (if ac_report.json unavailable)

### Coverage Determination

| Test Result | Coverage Status |
|-------------|-----------------|
| All steps passed | `passing` |
| Any step failed | `failing` |
| No test data | `unknown` |

---

## Documentation Validation

The `documentation` metrics validate documents against doc_type contracts:

| Doc Type | Validation Rules |
|-----------|----------------|
| `how_to` | Must reference at least one requirement or AC |
| `explanation` | Must reference at least one story or requirement |
| `design_doc` | Must reference at least one requirement |
| `reference` | Must reference at least one requirement or AC |
| `status` | Must reference both requirements and ACs |
| `adr` | Must reference at least one requirement |
| `guide` | Must reference at least one requirement or AC |
| `impl_plan` | Must reference both requirements and ACs |
| `requirements_doc` | Must reference at least one requirement |
| `ci_workflow` | No validation (YAML workflows) |

---

## Task Hints

The `task_hints` section provides prioritized work items for agents:

### High Priority Tasks

Top 5 tasks that are:
- Status: `open` or `in_progress`
- Sorted by: status (in_progress first), then priority label, then task ID

### Friction and Question Counts

- `friction_count`: Number of open friction entries
- `question_count`: Number of open questions

These counts help agents understand the volume of open issues.

---

## Use Cases

### For IDP Integration

The IDP snapshot is designed for Internal Developer Platforms to:

1. **Assess governance health** before deploying services
2. **Check AC coverage** for deployment gates
3. **Get prioritized work** for agent task queues
4. **Monitor documentation health** for knowledge base quality

### Example Integration Flow

```python
import requests

def get_platform_snapshot(platform_url: str) -> dict:
    """Get IDP snapshot from platform API."""
    response = requests.get(f"{platform_url}/platform/idp/snapshot")
    response.raise_for_status()
    return response.json()

def assess_deployment_readiness(snapshot: dict) -> bool:
    """Assess if service is ready for deployment."""
    if snapshot["governance_health"]["status"] != "healthy":
        return False
    if snapshot["governance_health"]["ac_coverage"]["failing"] > 0:
        return False
    return True

# Usage
snapshot = get_platform_snapshot("http://localhost:8080")
if assess_deployment_readiness(snapshot):
    print("Service is ready for deployment")
else:
    print("Service has governance issues")
```

---

## Related Endpoints

- [`/platform/status`](./status.md) - Full platform status with error tracking
- [`/platform/agent/hints`](./agent-hints.md) - Detailed agent hints with filtering
- [`/platform/coverage`](./overview/index.md) - Detailed AC coverage

---

## Related Documentation

- [Platform API Overview](./overview/index.md)
- [Platform API Reference](../reference/platform-api-endpoints.md)
- [IDP Cell Contract](../IDP_CELL_CONTRACT.md)
- [Agent Guide](../AGENT_GUIDE.md)
