---
id: REF-IDP-CELL-CONTRACT-001
title: IDP Cell Contract
doc_type: reference
status: published
audience: platform-engineers, idp-operators, integration-developers
tags: [idp, platform, kernel, contract, integration]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-PLATFORM-APIS, REQ-TPL-IDP-SNAPSHOT, REQ-TPL-AI-IDP-COMPAT]
acs: [AC-PLT-015, AC-TPL-IDP-SNAPSHOT, AC-TPL-IDP-SNAPSHOT-VALID-JSON, AC-TPL-IDP-CELL-SMOKE, AC-TPL-CLI-JSON-CORE]
adrs: [ADR-0004, ADR-0017]
last_updated: 2025-12-22
---

<!-- doclint:disable orphan-version -->

# IDP Cell Contract

> **For platform/IDP teams:** This document is the datasheet for integrating this Rust cell
> into your Internal Developer Platform. It defines what you can rely on, what to consume,
> and how to validate cell health.

**Kernel Version:** v3.3.9-kernel | **Template Version:** 3.3.14

---

## What This Cell Provides

This repository is a **governed Rust service cell** designed for IDP integration:

- **72+ kernel ACs** validated by `cargo xtask selftest` (run `cargo xtask ac-status --summary` for exact count)
- **Stable `/platform/*` introspection APIs** for governance health, tasks, and docs
- **Machine-readable contracts** via `cargo xtask idp-snapshot` and `kernel_contract.vX.Y.Z.json`
- **CI-enforced governance** via `tier1-selftest.yml` with strict coverage

An IDP can treat this cell as a **governed, observable service unit** that self-reports its health, coverage, and work items.

---

## The IDP Profile

The following ACs constitute the **IDP contract** - the stable surfaces an IDP can rely on:

### Core Introspection ACs

| AC | Surface | Description |
|----|---------|-------------|
| `AC-PLT-015` | `/platform/status` | Governance health, AC coverage, selftest gates |
| `AC-PLT-015` | `/platform/graph` | Full REQ/AC/test/doc relationship graph |
| `AC-PLT-015` | `/platform/docs/index` | Documentation inventory with health status |
| `AC-PLT-019` | `/platform/tasks` | Task management with status filtering |
| `AC-PLT-024` | `/platform/agent/hints` | Prioritized work suggestions for agents |
| `AC-TPL-PLATFORM-SCHEMA` | `/platform/schema` | Schema index for all platform endpoints |
| `AC-TPL-PLATFORM-SCHEMA` | `/platform/openapi` | OpenAPI spec for all platform endpoints |

### IDP-Specific ACs

| AC | Surface | Description |
|----|---------|-------------|
| `AC-TPL-IDP-SNAPSHOT` | `cargo xtask idp-snapshot` | Machine-readable IDP tile data |
| `AC-TPL-IDP-SNAPSHOT-VALID-JSON` | idp-snapshot output | Valid JSON with all required keys |
| `AC-TPL-CLI-JSON-CORE` | `--json` flag | CLI commands emit structured JSON |

### Governance Enforcement ACs

| AC | Surface | Description |
|----|---------|-------------|
| `AC-TPL-BDD-EXIT-CODES` | BDD harness | Deterministic exit codes for CI |
| `AC-TPL-GRAPH-REQ-HAS-AC` | Graph invariants | Every REQ has an AC |
| `AC-TPL-GRAPH-AC-HAS-TEST` | Graph invariants | Every AC has a test |

---

## Surfaces to Consume

### 1. IDP Snapshot (Primary)

The `idp-snapshot` command is the **primary integration point** for IDP tiles:

```bash
cargo xtask idp-snapshot --pretty
```

**Output structure:**

```json
{
  "timestamp": "2025-12-09T12:00:00Z",
  "template_version": "3.3.14",
  "service_id": "template-service",
  "governance_health": {
    "status": "healthy",
    "ac_coverage": {
      "total": 72,
      "passing": 72,
      "failing": 0,
      "unknown": 0
    },
    "spec_counts": {
      "stories": 8,
      "requirements": 25,
      "acceptance_criteria": 65
    }
  },
  "documentation": {
    "total": 45,
    "valid": 45,
    "with_issues": 0
  },
  "task_hints": {
    "total_pending": 3,
    "total_in_progress": 1,
    "friction_count": 2,
    "question_count": 1,
    "high_priority": [
      {
        "task_id": "TASK-001",
        "title": "Implement feature X",
        "status": "in_progress",
        "requirement_ids": ["REQ-001"],
        "ac_ids": ["AC-001"]
      }
    ]
  }
}
```

### 2. HTTP Introspection APIs

| Endpoint | Use Case | Response |
|----------|----------|----------|
| `GET /platform/status` | Health widget, dashboard tile | Governance metrics, selftest gates |
| `GET /platform/graph` | Dependency visualization | Stories/REQs/ACs/docs/commands |
| `GET /platform/docs/index` | Doc portal sync | Documentation inventory |
| `GET /platform/tasks` | Kanban board | Task list with filters |
| `GET /platform/agent/hints` | Agent orchestration | Prioritized work suggestions |
| `GET /platform/schema` | Schema discovery | JSON Schema index + endpoint list |
| `GET /platform/openapi` | Codegen, validation | OpenAPI specification |

### 3. CLI Commands (JSON Mode)

All governance commands support `--json` for scripting:

```bash
cargo xtask version --json        # Version info
cargo xtask ac-status --json      # AC coverage (v2.0 schema)
cargo xtask friction-list --json  # Friction log
cargo xtask fork-list --json      # Fork registry
cargo xtask kernel-status --json  # Cell readiness probe
```

### 4. Release Artifacts

| Artifact | Location | Description |
|----------|----------|-------------|
| Kernel contract | `release_evidence/kernel_contract.vX.Y.Z.json` | Machine-readable kernel ACs |
| Release evidence | `release_evidence/vX.Y.Z.md` | Human-readable release notes |
| AC status | `docs/feature_status.md` | AC health matrix |
| OpenAPI spec | `specs/openapi/openapi.yaml` | Platform API schema |

---

## How to Trust a Cell

A cell built on this template is considered **valid** when:

1. **`tier1-selftest` passes on main** - The authoritative CI gate
2. **`idp-snapshot` returns valid JSON** - Machine-readable health
3. **`kernel_contract.vX.Y.Z.json` exists** - Release evidence generated

### CI Enforcement

The `tier1-selftest.yml` workflow is the **only** workflow that enforces kernel contracts:

- Sets `XTASK_STRICT_AC_COVERAGE=1` on main branch
- Sets `XTASK_STRICT_PRECOMMIT=1` for strict mode
- Runs all 11 selftest steps (fmt, clippy, tests, BDD, policies, graph invariants)

Other workflows (`selftest.yml`, `ci-template-selftest.yml`) are convenience checks and do not enforce the full kernel contract.

### Local Validation

For local cell readiness validation:

```bash
cargo xtask kernel-status
```

This aggregates:
- Template/kernel version
- Git state (clean/dirty)
- Kernel AC coverage (must_have_ac=true ACs)
- Docs invariants
- CI gate configuration

---

## Versioning Model

| Version Type | Tag Pattern | Stability | Use Case |
|--------------|-------------|-----------|----------|
| **Kernel** | `vX.Y.Z-kernel` | Frozen contract | IDP integration (stable APIs) |
| **Template** | `vX.Y.Z` | May evolve | General development |

### What This Means

- **Target kernel tags** when you care about contract stability
- **Track template tags** for latest features and docs
- Kernel tags guarantee:
  - `must_have_ac=true` ACs won't change behavior
  - `/platform/*` response shapes are stable
  - xtask governance commands work as documented

### Changing the Kernel

Any change that affects kernel contracts requires:

1. ADR documenting the change
2. Version bump in `specs/spec_ledger.yaml`
3. Regenerate `release_evidence/*`
4. New `vX.Y.Z-kernel` tag

See [KERNEL_SNAPSHOT.md](./KERNEL_SNAPSHOT.md#changing-the-kernel-after-v337-kernel) for the full protocol.

---

## IDP Integration Patterns

### Pattern 1: Health Tile

```python
import requests

def get_cell_health(base_url: str) -> dict:
    status = requests.get(f"{base_url}/platform/status").json()
    return {
        "healthy": status["governance"]["selftest_status"] == "pass",
        "ac_coverage": f"{status['governance']['ac_pass']}/{status['governance']['ac_total']}",
        "version": status["service"]["version"]
    }
```

### Pattern 2: Scorecard Integration

```python
def get_cell_scorecard(base_url: str) -> dict:
    status = requests.get(f"{base_url}/platform/status").json()
    gov = status["governance"]

    return {
        "governance_score": int((gov["ac_pass"] / gov["ac_total"]) * 100),
        "selftest_gates": len([g for g in gov["selftest_gates"] if g["status"] == "pass"]),
        "total_gates": len(gov["selftest_gates"]),
        "policy_compliance": gov.get("policies_pass", 0) / gov.get("policies_total", 1) * 100
    }
```

### Pattern 3: Work Item Sync

```python
def sync_tasks_to_tracker(base_url: str, tracker):
    tasks = requests.get(f"{base_url}/platform/tasks").json()
    hints = requests.get(f"{base_url}/platform/agent/hints").json()

    for task in tasks["tasks"]:
        tracker.upsert(
            id=task["id"],
            title=task["title"],
            status=task["status"],
            requirement=task.get("requirement"),
            acs=task.get("acs", [])
        )

    # Add agent recommendations as metadata
    for hint in hints["hints"]:
        tracker.add_recommendation(hint["task_id"], hint["suggested_commands"])
```

### Pattern 4: Event-Driven Updates

```yaml
# GitHub webhook on tag push
on:
  push:
    tags:
      - 'v*.*.*-kernel'

jobs:
  update-idp:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Generate IDP snapshot
        run: cargo xtask idp-snapshot --output snapshot.json
      - name: Push to IDP registry
        run: |
          curl -X POST $IDP_REGISTRY_URL \
            -H "Content-Type: application/json" \
            -d @snapshot.json
```

---

## Referential Integrity

The agent hints API (`/platform/agent/hints`) and bundle command (`cargo xtask bundle`) validate
referential integrity of task definitions.

### Agent Hints Warnings

When a task references a non-existent AC or REQ in `specs/tasks.yaml`, the hints response includes a `warnings` array:

```json
{
  "hints": [...],
  "warnings": [
    {
      "invalid_id": "AC-NONEXISTENT-999",
      "ref_type": "ac",
      "source": "TASK-001",
      "message": "Task TASK-001 references non-existent AC AC-NONEXISTENT-999"
    }
  ]
}
```

This enables agents and IDPs to:
- Detect governance drift without blocking work
- Surface broken references for human review
- Prioritize fixing referential issues

### Bundle Warnings

Bundle generation logs warnings for invalid AC/REQ references:

```bash
cargo xtask bundle my_task
# Output includes: [WARN] AC-NONEXISTENT-999 not found in spec_ledger.yaml
```

Set `BUNDLE_STRICT_REFS=1` to fail on invalid references in CI:

```bash
BUNDLE_STRICT_REFS=1 cargo xtask bundle my_task
# Exits with error if any invalid references found
```

---

## Understanding AC Status

The `docs/feature_status.md` file shows AC execution status with three states:

| Status | Meaning |
|--------|---------|
| `[PASS]` | At least one test (BDD or unit) ran and passed |
| `[FAIL]` | At least one test ran and failed |
| `[UNKNOWN]` | No local test ran for this AC |

### Meta / CI-only ACs

Some ACs show `[UNKNOWN]` status by design. These are **meta/CI-only ACs**:

1. **CI-validated ACs** - Tested in GitHub Actions, not locally (e.g., `AC-TPL-EXAMPLE-FORK-BUILDS`)
2. **Policy ACs** - Define contracts validated by linting tools, not executable tests
3. **Documentation ACs** - Verified by `docs-check`, not BDD scenarios

This is **intentional**. The kernel contract (`must_have_ac=true` ACs) are all locally testable
and will show `[PASS]` or `[FAIL]`.

### IDP Implications

When integrating with an IDP:

- **Filter by `must_have_ac=true`** for core health metrics
- **Treat `[UNKNOWN]` as informational**, not a failure
- **Use `/platform/status` governance.selftest_status** as the authoritative health signal

---

## What's Out of Scope

This cell is a **per-service kernel**, not a full IDP. The following are intentionally left to the platform layer:

| Concern | Responsibility |
|---------|---------------|
| Authentication & authorization | Gateway/platform layer |
| SLOs and performance targets | Platform operators |
| Full observability stack | Platform pipelines |
| Org-level RBAC | Identity provider |
| Catalogue UIs | IDP/portal |
| Cross-service dependencies | Service mesh |

The cell contract stays **platform-agnostic**. Forks can add domain-specific REQs/ACs for these areas.

---

## Quick Reference

### Verify Cell Health

```bash
curl http://localhost:8080/platform/status | jq '.governance.selftest_status'
# Expected: "pass"
```

### Get IDP Tile Data

```bash
cargo xtask idp-snapshot --pretty
```

### Validate Cell Locally

```bash
cargo xtask kernel-status
```

### Check Kernel AC Coverage

```bash
cargo xtask ac-status --json | jq '.must_have_acs'
```

---

## See Also

- **[KERNEL_SNAPSHOT.md](./KERNEL_SNAPSHOT.md)** - Frozen kernel baseline
- **[integrate-idp-or-agent.md](./how-to/integrate-idp-or-agent.md)** - Integration recipes
- **[ci-workflows.md](./reference/ci-workflows.md)** - CI gate details
- **[idp-positioning.md](./explanation/idp-positioning.md)** - Relationship to IDPs
- **[TEMPLATE-CONTRACTS.md](./explanation/TEMPLATE-CONTRACTS.md)** - What the kernel guarantees
