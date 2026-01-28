---
id: HANDOVER-TPL-PLATFORM-CELL-001
title: Rust-as-Spec IDP Platform Cell - Handover
doc_type: guide
status: published
audience: maintainers, platform-engineers
tags: [handover, governance, microcrates, receipts, bdd, idp]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-ONBOARDING, REQ-PLT-DEVEX-CONTRACT, REQ-PLT-DOCS-CONSISTENCY]
acs: []
adrs: [ADR-0030]
last_updated: 2026-01-28
---
<!-- doclint:disable orphan-version -->
<!-- Note: This handover intentionally references kernel/template versions and PR numbers; they are historical context, not version_manifest-governed. -->

# Rust-as-Spec IDP Platform Cell - Handover

**Repo:** `EffortlessMetrics/Rust-Template`
**Template Version:** v3.3.14
**Kernel Baseline:** v3.3.9-kernel

This document is a maintainer takeover packet: what exists, what is contractual, how to validate, and what's left to push it "over the line" into a stable, repeatable, upgradeable platform cell.

---

## 1) Current State

### What's True Now

- The workspace is **microcrated** (~51 crates) with a contract-first taxonomy (contract/foundation/core/router/facade)
- `cargo xtask selftest` runs the full **12-step** gate and is **green** on `main`
- BDD features under `specs/features/` run via the `acceptance` crate
- Receipts are separated into:
  - **types**: `crates/receipts-core`
  - **generation/IO/validation**: `crates/xtask-receipts`
  - `xtask` is intended to be **wiring-only** (engines live in microcrates)

### Recent Improvements

- Selftest expanded from 11 to 12 steps (added Docs-as-Code check)
- `docs-check` now validates markdown links, BDD tags, and Skills/Agents alignment (advisory)
- `AppError` refactored to use boxed details (reduced stack size)
- `gov-http-friction` and `gov-http-questions` now use `spawn_blocking` for filesystem I/O
- Acceptance tests improved with proper `Result<()>` error handling

---

## 2) What This Repo Is (and Is Not)

This repo is a **governed service-cell template** intended to be:

- A per-service "truth surface" for an IDP/portal (Backstage/Port/etc.)
- A "factory" where specs/tests/docs/policies agree
- Agent-friendly (autonomous work is supported, but bounded by contracts and `selftest`)

It is **not** the portal/IDP. It is the "cell" the portal queries and CI can verify.

---

## 3) The Contracts That Matter

Treat these as **couplers**: rare, stable, boring. Make change loud.

### 1. Platform HTTP Contract

- `/platform/*` JSON payloads + `/platform/openapi`
- Additive is cheap; breaking is loud

### 2. CLI Contract

- `cargo xtask ...` command set + exit semantics
- `--json` outputs where offered

### 3. Spec Contract

- YAML shapes + semantics for `specs/*.yaml` (ledger, tasks, DevEx flows, docs index, config schema)

### 4. Receipts Contract

- Receipt JSON shapes (`gate.json`, `quality.json`, `telemetry.json`, `timeline.json`, etc.)
- Schema versioning discipline

**Rule:** if a portal/agent/automation consumes it, it's a contract surface and must be tripwired.

---

## 4) Microcrate Taxonomy

### Contract Crates (dependency-light; intended stable)

- `platform-contract` - DTOs + envelopes for `/platform/*`
- `spec-types` - IDs/newtypes/shared spec structs
- `receipts-core` - receipt schema types + version fields
- `xtask-contract` - stable DTOs for `--json` outputs (where used)

### Spec Engine Crates (load/validate/graph/tasks/hints)

- `spec-ledger`, `spec-graph`, `spec-docs`, `spec-tasks`, `spec-devex`, `spec-schema`
- `spec-ui-contract`, `spec-iac`, `spec-metadata`

### HTTP/Router Crates (Axum glue; domain routers)

- `http-core`, `http-errors`, `http-middleware`
- `http-platform` (platform endpoints + UI)
- `http-tasks`, `http-todos`, `http-agents`
- `gov-http-core` + `gov-http-{forks,friction,questions,issues}` + `gov-http` facade

### Tooling Engines (reusable)

- `xtask-lib` (repo/fs/process helpers)
- `xtask-versioning` (manifest-driven version engine)
- `xtask-contracts` (docs/contracts checks)
- `xtask-receipts` (receipt generation + validation + run_id)

### Facades / Assembly Crates (stable entrypoints)

- `app-http` - runtime assembly + facade exports
- `gov-http` - router facade
- `spec-runtime` - compatibility facade (re-exports while migration completes)
- `xtask` - CLI assembly/wiring

**Hard rule:** engines own behavior/types; assemblies own wiring.

---

## 5) Local Dev and Validation Ladder

### Tier-1 Environment (CI parity)

```bash
nix develop
cargo xtask dev-up
```

### Fast Loop

```bash
cargo xtask check
```

### Change-Aware Loop

```bash
cargo xtask test-changed
```

### Full Gate (arbiter)

```bash
cargo xtask selftest
```

### BDD Only (isolation)

```bash
cargo test -p acceptance --test acceptance
```

When something is red: don't guess. Re-run the ladder:
`check` -> `test-changed` -> targeted test -> `selftest`.

---

## 6) Running the Service and Inspecting the Cell

### Start

```bash
cargo run -p app-http
```

### Endpoints to Exercise

| Endpoint | Purpose |
|----------|---------|
| `/platform/status` | Governance health |
| `/platform/openapi` | Contract surface |
| `/platform/graph` | Governance graph |
| `/platform/idp/snapshot` | IDP tile payload |
| `/platform/docs/index` | Documentation inventory |
| `/platform/devex/flows` | Developer workflows |
| `/platform/issues` | Unified issues surface |
| `/ui` | Dashboard (+ `/ui/graph`, `/ui/flows`, `/ui/coverage`) |

### Auth Mode (if enabled)

- `PLATFORM_AUTH_MODE=basic`
- `PLATFORM_AUTH_TOKEN=...`
- Non-GET write endpoints enforce auth; read endpoints may remain open depending on policy

---

## 7) Receipts System

### Separation (important)

- **Schema/types:** `crates/receipts-core`
- **Generation/IO/validation:** `crates/xtask-receipts`
- `xtask` should call engines; it should not re-implement receipt logic

### Run Correlation

- `run_id` correlates a batch of receipts and supporting forensic artifacts across commands

### Outputs

Typical run dirs live under `.runs/...` with:

- `receipts/*.json`
- optional supporting artifacts (logs, diffs, etc.)

### Validation

Receipt JSON should validate against schema JSON (where present). Keep validation in engine crates.

---

## 8) Governance and Policy

### Scope Guard

- `policy/scope.rego` classifies PRs and enforces danger-zone declarations (often advisory first; tighten later)

### Truth Labels / Semantic-Only Merge Rules

- `.claude/rules/...` defines claim labeling and deterministic-vs-LLM separation

Intent: make "what changed" reviewable at speed without trusting narrative.

---

## 9) What's Left to Take This "Over the Line"

This punch list turns "working refactor" into "operational platform cell."

### A) Make CI Authoritative (not advisory)

Make these **required checks**:

- OpenAPI diff gate
- `--json` schema / golden diff gate
- public API diff for contract crates
- layering check: forbid `axum/tokio/clap/sqlx/...` in contract crates

### B) Finish Contract Versioning Discipline

- Enforce contract manifests (schema manifests, version manifest) in CI
- Make `release-prepare` atomic across workspace + docs

### C) Close "Release Safety" ACs

Implement missing parts, then promote from advisory -> enforced.

### D) Package for Adopters (real win)

Minimize boilerplate in adopter repos:

- meta crates:
  - `cell-runtime` (mount `/platform/*`, middleware defaults, load specs)
  - `cell-factory` (selftest/docs-check/receipts/policy)
- optional: a `cargo-cell` plugin so adopters don't vendor `xtask`

### E) Prove Adoption

- Spin a minimal real fork and capture friction
- Use that friction to drive the next kernel decisions

---

## 10) How to Extend Safely

### Add a New Platform Endpoint

1. Add DTOs to `platform-contract` (if on contract surface)
2. Implement handler in the correct router crate (`http-platform` or `gov-http-*`)
3. Update OpenAPI (if served from spec)
4. Add BDD scenario + tag + ledger mapping
5. Run `cargo xtask selftest`

### Add a New Receipt Type

1. Add schema types to `receipts-core`
2. Add generator + schema validation to `xtask-receipts`
3. Add `xtask` wiring command
4. Add golden/schema checks (if applicable)
5. Run `cargo xtask selftest`

### Add a New Microcrate

- contract-first (types in contract crates)
- engines own behavior
- assemblies re-export during migration
- keep public APIs tiny

---

## 11) "Start Here" Map

### Read First

- `docs/adr/0030-microcrate-architecture.md`
- `docs/contracts/contract-inventory.md`
- `docs/reference/xtask-commands.md`
- `docs/audit/AUDIT_PATH.md`

### Tooling + Governance

- `crates/xtask/src/...` - CLI assembly
- `crates/xtask-receipts/` - receipts engine
- `policy/scope.rego` - scope guard
- `.pre-commit-config.yaml` - precommit gates

### Runtime

- `crates/app-http/` - assembly
- `crates/http-platform/` - platform endpoints + UI
- `crates/gov-http-*/` - governance endpoints

### Specs

- `specs/spec_ledger.yaml`
- `specs/config_schema.yaml`
- `specs/devex_flows.yaml`
- `specs/tasks.yaml`
- `specs/openapi/openapi.yaml`

---

## 12) "First Hour" Takeover Script

```bash
git clone <repo>
cd Rust-Template
nix develop

cargo xtask doctor
cargo xtask selftest

cargo run -p app-http &
curl localhost:8080/platform/status | jq
curl localhost:8080/platform/openapi | head
curl localhost:8080/platform/idp/snapshot | jq
open http://localhost:8080/ui
```

Then read:

- ADR-0030
- contract inventory
- xtask commands reference

---

## Appendix: Selftest Steps (v3.3.14+)

The 12-step selftest gate validates:

1. **Core checks** - fmt, clippy, unit tests
2. **Docs-as-Code** - version alignment, markdown links, BDD tags
3. **Skills governance** - lint and validate Skills
4. **Agents governance** - lint and validate Agents
5. **BDD acceptance tests** - Cucumber scenarios
6. **AC/ADR mapping** - AC status and ADR references
7. **LLM bundler** - context bundle generation
8. **Policy tests** - OPA/Conftest validation
9. **DevEx contract** - required commands and flows
10. **Governance graph & UI** - graph invariants
11. **AC coverage** - kernel AC coverage check
12. **Test coverage** - advisory coverage reporting

<!-- doclint:enable orphan-version -->
