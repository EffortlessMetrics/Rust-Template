---
id: RUNBOOK-TPL-KERNEL-001
service: rust-as-spec-kernel
owner: team-platform
last_updated: 2025-11-20
---

# Platform Kernel Runbook

## Overview

The Rust-as-Spec platform kernel provides introspection endpoints for governed services. It exposes the complete governance graph (stories, requirements, ACs, docs, commands, flows) via HTTP endpoints.

## Architecture

```
/platform/graph       → Complete governance graph (JSON)
/platform/devex/flows → DevEx commands and flows (JSON)
/platform/docs/index  → Document index (JSON)
/platform/status      → Governance health status (JSON)
```

## Endpoints

### `GET /platform/graph`

Returns the complete governance graph including:
- Stories and requirements
- Acceptance criteria
- Documents (design, plans, runbooks)
- DevEx commands
- Flows

**Use case:** Agents and dashboards query this to understand service structure and traceability.

**Example:**
```bash
curl http://localhost:8080/platform/graph
```

### `GET /platform/devex/flows`

Returns DevEx flows and commands from `specs/devex_flows.yaml`.

**Use case:** Tools discover how to operate on the repository (e.g., "what commands are available?", "what's the release flow?").

**Example:**
```bash
curl http://localhost:8080/platform/devex/flows
```

### `GET /platform/docs/index`

Returns the document index from `specs/doc_index.yaml`, mapping docs to the stories/requirements/ACs/ADRs they cover.

**Use case:** Find design docs, ADRs, and requirements documentation.

**Example:**
```bash
curl http://localhost:8080/platform/docs/index
```

### `GET /platform/status`

Returns high-level governance health status.

**Example:**
```bash
curl http://localhost:8080/platform/status
```

## Kernel Contract

- **Kernel AC set:** Tracked in `specs/spec_ledger.yaml` and `docs/feature_status.md` (all kernel ACs are ✅ via `cargo xtask selftest`).
  - DevEx: AC-PLT-001/003/008/014/018 (doctor, check, sbom-local, devex flows, dev-up)
  - Observability/Core: AC-TPL-001/002/003/004/007 (health, version, error envelope, metrics)
  - Governance Graph: AC-TPL-GRAPH-REQ-HAS-AC / AC-HAS-TEST / COMMAND-REACHABLE / SELFTEST / MERMAID
  - Tasks + Agent: AC-TPL-TASKS-CLI/CREATE-CLI/UPDATE-CLI/HTTP, AC-TPL-AGENT-SKILLS, AC-TPL-AGENT-HINTS
- **Required xtask commands (Tier-1 gate):**
  - `cargo xtask selftest` (single gate; runs check + BDD + ac-status + graph invariants)
  - `cargo xtask check` (fmt/clippy/tests + change-aware BDD)
  - `cargo xtask test-changed --plan-only` (selective testing ladder)
  - `cargo xtask test-ac <AC-ID>` (targeted AC execution)
  - `cargo xtask ac-status` (AC ↔ test mapping; reads BDD + unit results)
  - `cargo xtask graph-export --check` (graph invariants)
  - `cargo xtask tasks-list | task-create | task-update` (tasks surfacing CLI)
- **Required platform endpoints:**
  - `/platform/status`, `/platform/graph`, `/platform/devex/flows`, `/platform/docs/index`
  - `/platform/tasks` (list/filter) and `/platform/tasks/{id}/status` (update)
  - `/platform/agent/hints` (agent-facing task suggestions)
- **Metadata & release expectations:**
  - `specs/service_metadata.yaml` template_version: `3.2.0`, links to ROADMAP, kernel contract, AGENT_GUIDE, SELECTIVE_TESTING, platform support.
  - Release bundles via `cargo xtask release-bundle <version>` capture selftest, AC status, tasks, and policies.
  - Governance evidence lives in `release_evidence/` and `FRICTION_LOG.md`.

## Troubleshooting

### Graph endpoint returns 500

**Symptoms:** `/platform/graph` returns internal server error

**Cause:** Spec files are invalid or missing

**Resolution:**
```bash
cargo xtask docs-check
cargo xtask selftest
```

### Missing data in graph

**Symptoms:** Graph is missing expected nodes/edges

**Cause:** Spec files incomplete or loaders not picking up all data

**Resolution:**
Verify specs are complete:
```bash
cargo xtask graph-export
cargo run -p xtask -- ac-status
```

### Service won't start

**Symptoms:** Service fails to bind to port or crashes on startup

**Cause:** Port conflict or missing environment variables

**Resolution:**
```bash
# Check if port is in use
lsof -i :8080

# Run with different port
SERVICE_PORT=8081 cargo run
```

## Runbook Maintenance

This runbook should be updated whenever:
- New platform endpoints are added
- Spec schemas change
- Common troubleshooting patterns emerge

Last reviewed: 2025-11-20

