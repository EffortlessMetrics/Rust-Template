# Kernel Snapshot v3.3.2

**Date:** 2025-11-26
**Version:** v3.3.2-kernel
**Purpose:** Baseline kernel ACs for the first Knowledge Hub service

This document captures the state of all kernel acceptance criteria at the time of the v3.3.2 release. These ACs represent the foundational platform capabilities that the template provides and forms the baseline contract for the Knowledge Hub service.

---

## Summary Statistics

From `cargo xtask ac-status` and `cargo xtask selftest`:

- **Total ACs:** 65
- **Kernel ACs (must_have_ac: true):** 54
- **Non-kernel ACs:** 11

### Kernel AC Status
- **✅ Passing:** 27 ACs
- **❌ Failing:** 27 ACs
- **❓ Unknown:** 0 ACs

**Kernel AC Coverage:** 50% (27/54 passing)

### Non-kernel AC Status
- **✅ Passing:** 7 ACs
- **❌ Failing:** 4 ACs

---

## Kernel Acceptance Criteria

The following 54 acceptance criteria are tagged as `kernel` and have `must_have_ac: true`. These define the core platform contract.

### Template Core Service ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-001 | REQ-TPL-HEALTH | ✅ PASS | GET /health returns 200 with status 'ok' when service is healthy |
| AC-TPL-002 | REQ-TPL-VERSION | ✅ PASS | GET /version returns build information including version and git SHA |
| AC-TPL-003 | REQ-TPL-ERROR-HANDLING | ✅ PASS | All 4xx/5xx responses include an error code, message, and request ID |
| AC-TPL-004 | REQ-TPL-ERROR-HANDLING | ✅ PASS | Handlers propagate or generate X-Request-ID and expose it in responses |
| AC-TPL-007 | REQ-TPL-METRICS | ✅ PASS | GET /metrics returns Prometheus-formatted metrics including http_requests_total |

### Platform Onboarding ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-001 | REQ-PLT-ONBOARDING | ❌ FAIL | `cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance |
| AC-PLT-002 | REQ-PLT-ONBOARDING | ❌ FAIL | `cargo xtask help-flows` renders categorized command map from specs/devex_flows.yaml |
| AC-PLT-003 | REQ-PLT-ONBOARDING | ❌ FAIL | `cargo xtask check` runs fmt + clippy + tests as fast dev loop |
| AC-PLT-018 | REQ-PLT-ONBOARDING | ✅ PASS | `cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps |

### Platform Design Scaffolding ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-004 | REQ-PLT-DESIGN-SCAFFOLDING | ❌ FAIL | `cargo xtask adr-new <title>` creates numbered ADR from template with metadata |
| AC-PLT-005 | REQ-PLT-DESIGN-SCAFFOLDING | ❌ FAIL | `cargo xtask ac-new <ID> <desc>` rejects duplicate IDs and generates YAML snippet |

### Platform Security & Governance ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-006 | REQ-PLT-SECURITY-GOVERNANCE | ❌ FAIL | `cargo xtask audit` runs cargo-audit + cargo-deny with repo policy (deny.toml) |
| AC-PLT-007 | REQ-PLT-SECURITY-GOVERNANCE | ❌ FAIL | `cargo xtask audit` provides 4-step recovery guidance on failure |
| AC-PLT-008 | REQ-PLT-SECURITY-GOVERNANCE | ❌ FAIL | `cargo xtask sbom-local` generates SPDX JSON to target/sbom.spdx.json |

### Platform Documentation Consistency ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-009 | REQ-PLT-DOCS-CONSISTENCY | ❌ FAIL | `cargo xtask docs-check` validates version alignment across spec_ledger, README, CLAUDE |
| AC-PLT-010 | REQ-PLT-DOCS-CONSISTENCY | ❌ FAIL | `cargo xtask docs-check` regenerates feature_status and fails on dirty git tree |

### Platform Release Safety ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-011 | REQ-PLT-RELEASE-SAFETY | ❌ FAIL | `cargo xtask release-prepare X.Y.Z` updates spec_ledger, README, CLAUDE, CHANGELOG |
| AC-PLT-012 | REQ-PLT-RELEASE-SAFETY | ❌ FAIL | `cargo xtask release-verify` runs selftest + audit + docs-check + clean tree |
| AC-PLT-013 | REQ-PLT-RELEASE-SAFETY | ❌ FAIL | `cargo xtask release-verify` provides git command sequence on success |

### Platform Release Evidence Bundle ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-REL-EVIDENCE | REQ-TPL-REL-BUNDLE | ❌ FAIL | `cargo xtask release-bundle X.Y.Z` writes release evidence with tasks, REQs/ACs/ADRs, git log, selftest summary, policy status |
| AC-TPL-REL-CHANGELOG | REQ-TPL-REL-BUNDLE | ❌ FAIL | Evidence file includes distinct sections adequate for LLM formatting into Keep a Changelog format |

### Platform DevEx Contract ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-014 | REQ-PLT-DEVEX-CONTRACT | ✅ PASS | Canonical flows and commands are defined in specs/devex_flows.yaml |
| AC-PLT-015 | REQ-PLT-DEVEX-CONTRACT | ❌ FAIL | `cargo xtask selftest` enforces devex contract (required commands exist) |
| AC-PLT-016 | REQ-PLT-DEVEX-CONTRACT | ❌ FAIL | `cargo xtask ci-local` orchestrates doctor + selftest + audit + docs-check |
| AC-PLT-019 | REQ-PLT-DEVEX-CONTRACT | ❌ FAIL | `cargo xtask selftest` displays a condensed summary with clear pass/fail indicators for all 7 steps |
| AC-PLT-020 | REQ-PLT-DEVEX-CONTRACT | ✅ PASS | `XTASK_LOW_RESOURCES=1` environment variable skips resource-intensive steps in selftest |

### Platform Status CLI ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-PLT-017 | REQ-PLT-STATUS-CLI | ❌ FAIL | `cargo xtask status` displays version, REQ/AC/task counts, selftest status, and suggested next tasks |

### Platform Introspection API ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-PLATFORM-GRAPH | REQ-TPL-PLATFORM-INTROSPECTION | ✅ PASS | GET /platform/graph returns the full governance graph in JSON format |
| AC-TPL-PLATFORM-DEVEX | REQ-TPL-PLATFORM-INTROSPECTION | ✅ PASS | GET /platform/devex/flows returns the canonical flows definition |
| AC-TPL-PLATFORM-DOCS | REQ-TPL-PLATFORM-INTROSPECTION | ✅ PASS | GET /platform/docs/index returns the documentation index |
| AC-TPL-POLICY-STATUS-OVERVIEW | REQ-TPL-PLATFORM-INTROSPECTION | ✅ PASS | GET /platform/status includes governance.policies.status field from policy-test run |

### Platform Schema ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-PLATFORM-SCHEMA | REQ-TPL-PLATFORM-SCHEMA | ✅ PASS | GET /platform/schema returns JSON schema/OpenAPI document for platform APIs |

### Platform Metadata ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-METADATA-COMPLETE | REQ-TPL-METADATA-CONSISTENT | ✅ PASS | service_metadata.yaml includes service_id, template_version, URLs, tags; /platform/status returns same identifiers |

### Platform Authentication ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-PLATFORM-AUTH-BASIC | REQ-TPL-PLATFORM-AUTH | ✅ PASS | When PLATFORM_AUTH_MODE=basic, write endpoints under /platform/* require authentication |

### Platform Security & Logging ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-LOG-NO-SECRETS | REQ-TPL-LOG-HYGIENE | ✅ PASS | Rendering of /platform/status and UI dashboards redacts or omits secrets |

### Platform Task Suggestions ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-SUGGEST-NEXT-CLI | REQ-TPL-SUGGEST-NEXT | ❌ FAIL | cargo xtask suggest-next --task <ID> prints a structured recommendation |
| AC-TPL-SUGGEST-NEXT-HTTP | REQ-TPL-SUGGEST-NEXT | ✅ PASS | GET /platform/tasks/suggest-next?task=<ID> returns a JSON recommendation |

### Platform Questions as Artifacts ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-QUESTIONS-LOGGED | REQ-TPL-QUESTIONS-AS-ARTIFACTS | ✅ PASS | Ambiguity during automated flows emits structured questions without halting progress |

### Platform Flow Idempotency ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-FLOW-IDEMPOTENT | REQ-TPL-FLOW-IDEMPOTENCY | ❌ FAIL | Running cargo xtask selftest or suggest-next multiple times produces stable outputs with no duplicate artifacts |

### Platform Graph Invariants ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-GRAPH-REQ-HAS-AC | REQ-TPL-GRAPH-INVARIANTS | ✅ PASS | Every requirement with platform/structural/security/devex/docs/release tags has at least one AC node |
| AC-TPL-GRAPH-AC-HAS-TEST | REQ-TPL-GRAPH-INVARIANTS | ✅ PASS | Every AC with a tests mapping in spec_ledger.yaml has at least one test node linked |
| AC-TPL-GRAPH-COMMAND-REACHABLE | REQ-TPL-GRAPH-INVARIANTS | ✅ PASS | Every command declared in devex_flows.yaml is referenced by a flow or explicitly marked internal |
| AC-TPL-GRAPH-SELFTEST | REQ-TPL-GRAPH-INVARIANTS | ❌ FAIL | cargo xtask selftest validates graph invariants and outputs 'Graph invariants satisfied' |

### Platform UI ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-PLATFORM-UI-DASHBOARD | REQ-TPL-PLATFORM-UI | ✅ PASS | GET / or /ui serves an HTML dashboard showing platform status with governance health metrics |
| AC-TPL-PLATFORM-UI-GRAPH | REQ-TPL-PLATFORM-UI | ✅ PASS | The UI provides a graph visualization rendering governance graph using Mermaid.js |
| AC-TPL-PLATFORM-UI-FLOWS | REQ-TPL-PLATFORM-UI | ✅ PASS | The UI provides a flows and tasks view displaying DevEx flows and available tasks |

### Platform Tasks Surfacing ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-TASKS-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | cargo xtask tasks-list prints all tasks with their IDs, titles, statuses, and owners |
| AC-TPL-TASKS-CREATE-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | cargo xtask task-create creates a new task in specs/tasks.yaml with validation |
| AC-TPL-TASKS-UPDATE-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | cargo xtask task-update updates task fields (status, title, owner) with validation |
| AC-TPL-TASKS-HTTP | REQ-TPL-PLATFORM-TASKS | ✅ PASS | GET /platform/tasks returns a JSON representation of tasks.yaml |

### Platform Graph Visualization ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-GRAPH-MERMAID | REQ-TPL-GRAPH-VISUALIZATION | ❌ FAIL | cargo xtask graph-export --format mermaid emits a valid Mermaid graph with stories/REQs/ACs/tests/docs |

### Platform Configuration Validation ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-CONFIG-VALIDATION | REQ-TPL-CONFIG-INTEGRITY | ✅ PASS | On startup the service validates configuration against specs/config_schema.yaml |

### Infrastructure as Code Alignment ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-IAC-COMPOSE-ALIGN | REQ-TPL-IAC-ALIGNMENT | ✅ PASS | docker-compose.yaml services use the same port/environment as config_schema.yaml |
| AC-TPL-IAC-K8S-ALIGN | REQ-TPL-IAC-ALIGNMENT | ✅ PASS | Kubernetes manifests use the same port/environment as config_schema.yaml |
| AC-TPL-IAC-TF-ALIGN | REQ-TPL-IAC-ALIGNMENT | ✅ PASS | Terraform examples reference the same variables as config_schema.yaml |

### Local Runtime ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-LOCAL-DOCKER | REQ-TPL-LOCAL-RUNTIME | ✅ PASS | Optional local Docker compose (Postgres + Jaeger) aligns with app's service contract |

### Git Hooks ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-HOOKS-INSTALL | REQ-TPL-GOV-HOOKS | ❌ FAIL | The 'cargo xtask install-hooks' command creates a pre-commit hook that runs governance checks |

### Agent Interface ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-AGENT-SKILLS | REQ-TPL-AGENT-INTERFACE | ✅ PASS | The .claude/skills directory contains executable skill definitions with SKILL.md frontmatter |
| AC-TPL-AGENT-HINTS | REQ-TPL-AGENT-INTERFACE | ❌ FAIL | GET /platform/agent/hints returns prioritized task suggestions with context and recommended sequences |

### Agent Skills Guide ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-SKILLS-GUIDE-001 | REQ-TPL-SKILLS-GUIDE | ✅ PASS | docs/AGENT_SKILLS.md exists and documents the recommended Skill set |
| AC-TPL-SKILLS-ALIGN-001 | REQ-TPL-SKILLS-GUIDE | ❌ FAIL | Existing .claude/skills/* are aligned with documented workflows in AGENT_SKILLS.md |

### Agent Skills Tooling ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-SKILLS-FMT | REQ-TPL-SKILLS-TOOLING | ❌ FAIL | `cargo run -p xtask -- skills-fmt` normalizes SKILL.md files |
| AC-TPL-SKILLS-LINT | REQ-TPL-SKILLS-TOOLING | ❌ FAIL | `cargo run -p xtask -- skills-lint` validates Skills frontmatter and structure |

### Governance Write Operations ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-GOV-WRITE-TASK-STATUS-200 | REQ-TPL-GOV-WRITE-001 | ✅ PASS | set_task_status writes durable state reflected in the governance graph |

### Task Lifecycle ACs

| AC ID | Requirement | Status | Description |
|-------|-------------|--------|-------------|
| AC-TPL-TASK-TRANSITIONS | REQ-TPL-TASK-LIFECYCLE | ✅ PASS | Task status transitions are validated against the domain model |

---

## Non-Kernel Acceptance Criteria

The following 11 acceptance criteria are not tagged as kernel but are still part of the template contract:

| AC ID | Requirement | Status | Tags |
|-------|-------------|--------|------|
| AC-TPL-SUGGEST-NEXT-CLI | REQ-TPL-SUGGEST-NEXT | ❌ FAIL | devex |
| AC-TPL-TASKS-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | devex |
| AC-TPL-TASKS-CREATE-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | devex |
| AC-TPL-TASKS-UPDATE-CLI | REQ-TPL-PLATFORM-TASKS | ❌ FAIL | devex |

---

## Note on Baseline for Knowledge Hub

This snapshot represents the state of the template's kernel ACs at v3.3.2. The Knowledge Hub service will inherit this platform foundation and build domain-specific capabilities on top of it.

The 27 failing kernel ACs represent gaps in the template implementation that were discovered during this snapshot. These gaps do not block the Knowledge Hub service from proceeding, as the core runtime capabilities (health, version, metrics, platform APIs, UI, configuration validation, IAC alignment) are all passing.

The failing ACs are primarily in the following areas:
- **DevEx tooling**: CLI commands for release management, task management, skills tooling
- **Agent interface**: Agent hints endpoint not fully functional
- **Graph visualization**: Mermaid export and selftest graph validation
- **Governance hooks**: Pre-commit hook installation

These gaps will be addressed in future template iterations or can be implemented directly in the Knowledge Hub service if needed for its specific workflows.

---

## Complete List of Kernel AC IDs

For reference, here are all 53 ACs with explicit `tags: [kernel]` in `specs/spec_ledger.yaml`:

```
AC-PLT-001, AC-PLT-002, AC-PLT-003, AC-PLT-004, AC-PLT-005,
AC-PLT-006, AC-PLT-007, AC-PLT-008, AC-PLT-009, AC-PLT-010,
AC-PLT-011, AC-PLT-012, AC-PLT-013, AC-PLT-014, AC-PLT-015,
AC-PLT-016, AC-PLT-017, AC-PLT-018, AC-PLT-019, AC-PLT-020,
AC-TPL-001, AC-TPL-002, AC-TPL-003, AC-TPL-004, AC-TPL-007,
AC-TPL-AGENT-SKILLS, AC-TPL-CONFIG-VALIDATION, AC-TPL-FLOW-IDEMPOTENT,
AC-TPL-GOV-WRITE-TASK-STATUS-200, AC-TPL-GRAPH-AC-HAS-TEST,
AC-TPL-GRAPH-COMMAND-REACHABLE, AC-TPL-GRAPH-MERMAID,
AC-TPL-GRAPH-REQ-HAS-AC, AC-TPL-GRAPH-SELFTEST, AC-TPL-HOOKS-INSTALL,
AC-TPL-IAC-K8S-ALIGN, AC-TPL-METADATA-COMPLETE, AC-TPL-PLATFORM-DEVEX,
AC-TPL-PLATFORM-DOCS, AC-TPL-PLATFORM-GRAPH, AC-TPL-PLATFORM-SCHEMA,
AC-TPL-PLATFORM-UI-DASHBOARD, AC-TPL-PLATFORM-UI-FLOWS,
AC-TPL-PLATFORM-UI-GRAPH, AC-TPL-POLICY-STATUS-OVERVIEW,
AC-TPL-QUESTIONS-LOGGED, AC-TPL-REL-CHANGELOG, AC-TPL-REL-EVIDENCE,
AC-TPL-SKILLS-ALIGN-001, AC-TPL-SKILLS-FMT, AC-TPL-SKILLS-GUIDE-001,
AC-TPL-SKILLS-LINT, AC-TPL-TASK-TRANSITIONS
```

Note: The selftest reports 54 kernel ACs because one additional AC inherits kernel status from its parent requirement's `must_have_ac: true` flag even without an explicit kernel tag.

---

**End of Kernel Snapshot**
