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
/platform/graph       -> Complete governance graph (JSON)
/platform/devex/flows -> DevEx commands and flows (JSON)
/platform/docs/index  -> Document index (JSON)
/platform/status      -> Governance health status (JSON)
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

- **Kernel ACs:** Tracked in `specs/spec_ledger.yaml` and `docs/feature_status.md`; all kernel ACs are green on Tier-1 via `cargo xtask selftest`.
- **Optional ACs (non-kernel):** `AC-TPL-LOCAL-DOCKER` covers local Docker compose (Postgres + Jaeger) as a convenience. It is not gating and may remain `UNKNOWN`.
- **Policy checks:** `conftest`-backed policy tests are required in CI/Tier-1 where `conftest` is installed. Locally, selftest skips policies with a warning if `conftest` is missing; skipped policies do not fail the kernel.
- **ADR signals:** ADR coverage warnings are advisory only; no kernel AC currently fails on missing ADR references.
- **Key xtask commands (Tier-1 gate):**
  - `cargo xtask selftest` (single gate; runs check + BDD + ac-status + graph invariants)
  - `cargo xtask check` (fmt/clippy/tests + change-aware BDD)
  - `cargo xtask test-changed --plan-only` (selective testing ladder)
  - `cargo xtask test-ac <AC-ID>` (targeted AC execution)
  - `cargo xtask ac-status` (AC to test mapping; reads BDD + unit results)
  - `cargo xtask graph-export --check` (graph invariants)
  - Task CLI (`tasks-list`, `task-create`, `task-update`) exists and is green but is not a kernel gate.
- **Required platform endpoints:** `/platform/status`, `/platform/graph`, `/platform/devex/flows`, `/platform/docs/index` are kernel. Task/agent endpoints are implemented and green but remain non-kernel.
- **Metadata and release expectations:**
  - `specs/service_metadata.yaml` links to ROADMAP, kernel contract, AGENT_GUIDE, SELECTIVE_TESTING, platform support.
  - Release bundles via `cargo xtask release-bundle <version>` capture selftest, AC status, tasks, and policies.
  - Governance evidence lives in `release_evidence/` and `FRICTION_LOG.md`.

## Environment stance

- **Canonical environment:** Nix development shell on Linux/macOS/WSL2. This is the enforcement environment for the kernel contract.
- **Non-Nix Linux/macOS (bare host):** supported when toolchain parity is maintained (`rustup` + required tools installed). Policy checks (Conftest) are required in CI; locally they are best-effort and may be skipped with a clear warning.
- **Local Docker (optional, non-kernel):** Docker compose (`docker-compose.yaml`) is provided as ergonomic fallback for Postgres + Jaeger. `AC-TPL-LOCAL-DOCKER` is explicitly non-kernel and may remain `UNKNOWN` in local checks; teams may add a light unit/test to flip it to `[PASS]` without promoting it to a gating requirement.
- **Windows (Tier-2):** native Windows is supported for contributor workflows but is not the canonical enforcement surface; WSL2 + Nix is treated as Tier-1. Low-resource Windows runs may set `XTASK_LOW_RESOURCES=1` and skip heavy BDD steps.

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
