# Kernel Snapshot v3.3.3

**Date:** 2025-11-26 | **Version:** v3.3.3-kernel

## Executive Summary

This is the frozen kernel baseline (v3.3.3-kernel) for the Rust-as-Spec platform template. All 65 acceptance criteria pass. All 8 selftest gates pass. Day-0 commands work as documented. This is a stable, forkable baseline.

**Note:** "Selftest green" means the template meets its own specifications. It does not mean every use case has been validated in production. See [ROADMAP.md](./ROADMAP.md) for known gaps.

---

## Statistics

| Category | Count | Status |
|----------|-------|--------|
| **Total ACs** | 81 | All passing |
| **Kernel ACs** | 52 | All passing |
| **Non-kernel ACs** | 29 | All passing |
| **Selftest Gates** | 8 | All passing |

### Key Kernel Contracts (v3.3.3)

**Philosophy & Governance:**

- AC-TPL-BDD-EXIT-CODES: BDD harness exit codes distinguish test failures from skipped tests
- AC-TPL-FORKS-STATUS-SUMMARY: Fork registry visible in /platform/status
- AC-TPL-ARTIFACTS-HAVE-REFS: Governance artifacts support refs field for REQ/AC traceability
- AC-TPL-CLI-JSON-CORE: version and ac-status support --json for AI/IDP integration

**Environment & Testing:**

- AC-PLT-001: `cargo xtask doctor` validates Rust, Nix, conftest, git
- AC-PLT-020: BDD test execution and reporting (`cargo xtask bdd`)
- AC-PLT-NIX-DEVSHELL: Nix devshell provides reproducible environment

**Platform APIs:**

- AC-PLT-015: Platform introspection endpoints (/platform/status, /graph, /docs/index)
- AC-PLT-019: Task management via HTTP (GET /tasks, PATCH /tasks/{id}/status)
- AC-TPL-PLATFORM-GOVERNANCE-APIS: Governance APIs (/friction, /questions, /forks)

### Kernel vs Template Defaults

This table clarifies which ACs are **enforced kernel contracts** (must stay `must_have_ac: true`) vs **template defaults** (enabled here but forks can demote via `must_have_ac: false`):

| Category | Kernel ACs (Enforced) | Template Defaults (Can Demote) |
|----------|----------------------|-------------------------------|
| JSON CLI | AC-TPL-CLI-JSON-CORE | AC-TPL-CLI-JSON-OUTPUT |
| Artifacts & refs | AC-TPL-ARTIFACTS-HAVE-REFS | – |
| Fork visibility | AC-TPL-FORKS-STATUS-SUMMARY | – |
| Governance artifacts | – | AC-TPL-GOV-FRICTION, AC-TPL-GOV-FORKS |
| BDD harness | AC-TPL-BDD-EXIT-CODES | – |

**Override path:** See [docs/how-to/change-template-opinion.md](./how-to/change-template-opinion.md) for detailed instructions on promoting or demoting ACs in your fork

### Philosophy Contracts (Summary)

These REQs encode the template's opinionated stance. They define *how* the platform cell behaves, not just *what* it does:

| REQ ID | What It Enforces | Default ACs |
|--------|------------------|-------------|
| REQ-TPL-OPINIONATED-DEFAULTS | Opinions live as testable ACs, not tribal knowledge | AC-TPL-OPINIONS-DOCUMENTED |
| REQ-TPL-OVERRIDE-PATH | Forks change ACs/tests, not CI hacks | AC-TPL-OVERRIDE-DOC, AC-TPL-OVERRIDE-TRACEABLE |
| REQ-TPL-AI-IDP-COMPAT | AI/IDP surfaces are first-class, not afterthoughts | AC-TPL-CLI-JSON-CORE, AC-TPL-PLATFORM-GOVERNANCE-APIS |
| REQ-TPL-BDD-HARNESS | Deterministic exit codes for CI and agents | AC-TPL-BDD-EXIT-CODES |
| REQ-TPL-FLOW-IDEMPOTENCY | Platform flows are safe to rerun | AC-TPL-FLOW-IDEMPOTENT |
| REQ-TPL-GRAPH-INVARIANTS | Every REQ has an AC, every AC has a test | AC-TPL-GRAPH-REQ-HAS-AC, AC-TPL-GRAPH-AC-HAS-TEST |

**Why this matters:** When you fork, these philosophy contracts carry over. If you want different behavior (e.g., no JSON CLI requirement), you explicitly demote the AC in your fork's `spec_ledger.yaml`. This keeps "what we enforce" visible in spec, not buried in CI config.

**Cross-reference:** [`docs/how-to/pre-fork-checklist.md`](./how-to/pre-fork-checklist.md) → Phase 5: Choose Your Opinionation Level

---

## Key Capabilities

**Runtime & APIs:**

- Service health, version, metrics endpoints
- Platform introspection APIs:
  - `/platform/graph` - Full governance graph (REQ/AC/test/doc relationships)
  - `/platform/devex/flows` - Canonical DevEx flows and commands
  - `/platform/docs/index` - Documentation inventory
  - `/platform/schema` - OpenAPI contract
  - `/platform/status` - Governance health metrics (includes policy status)
- Task management APIs:
  - `/platform/tasks` - Task listing with filters (status, requirement)
  - `/platform/tasks/{id}/status` - Update task status via HTTP
  - `/platform/agent/hints` - Prioritized task suggestions for agents
- Friction & questions APIs:
  - `/platform/friction` - Friction log entries with statistics
  - `/platform/friction/{id}` - Individual friction entry details
  - `/platform/questions` - Question artifacts with status filtering
  - `/platform/questions/{id}` - Individual question details
- Fork tracking APIs:
  - `/platform/forks` - Fork registry with all registered forks
  - `/platform/forks/{name}` - Individual fork entry details
- Platform UI dashboard:
  - Interactive graph visualization using Mermaid.js
  - Flows and tasks view
  - Platform health status
- Configuration validation and IAC alignment (Docker Compose, Kubernetes, Terraform)
- Task lifecycle and governance write operations

**DevEx CLI:**

- Development flows:
  - `doctor` - Environment validation (Rust, Nix, conftest, git)
  - `help-flows` - Flow-based command map from specs/devex_flows.yaml
  - `check` - Fast dev loop (fmt + clippy + tests)
  - `test-changed` - Run tests affected by git changes
  - `ac-status` - AC coverage report (supports --json)
  - `ac-coverage` - Show which ACs need BDD scenarios
  - `dev-up` - One-command bootstrap (doctor + install-hooks + check)
- Bundler & agent tools:
  - `bundle` - Generate LLM context bundles for tasks
  - `suggest-next` - Task-aware next-step suggestions
- Governance:
  - `adr-new` - Create numbered ADR from template
  - `ac-new` - Create new acceptance criterion with validation
  - `docs-check` - Validate version alignment across spec_ledger, README, CLAUDE
  - `graph-export` - Export dependency graph (Mermaid format)
  - `selftest` - Full template self-test suite (8 gates)
  - `kernel-smoke` - Quick validation (docs-check + selftest)
  - `install-hooks` - Install pre-commit governance hooks
  - `skills-fmt` - Format Agent Skills (SKILL.md)
  - `skills-lint` - Lint Agent Skills (SKILL.md)
- Artifact management:
  - `friction-new` - Create new friction log entry
  - `friction-list` - List friction entries (with filters, supports --json)
  - `question-new` - Create new question artifact
  - `questions-list` - List questions from questions/ directory (supports --json)
- Fork tracking:
  - `fork-register` - Register a new template fork
  - `fork-list` - List registered forks (supports --json)
  - `version` - Show kernel/template version (supports --json)
- Task management:
  - `tasks-list` - List tasks from specs/tasks.yaml
  - `task-create` - Create new task with validation
  - `task-update` - Update task fields (status, title, owner)
- Release management:
  - `release-prepare` - Bump versions, update changelog
  - `release-bundle` - Generate release evidence with AC deltas
  - `release-verify` - Full release validation (selftest + audit + docs-check)
- Operational:
  - `audit` - Security audit (cargo-audit + cargo-deny)
  - `sbom-local` - Generate SPDX JSON SBOM
  - `ci-local` - Full CI simulation (doctor + selftest + audit + docs-check)
  - `status` - Governance status dashboard
  - `config-validate` - Validate config schema for an environment

**Governance:**

- BDD acceptance tests (110 scenarios passing, 65 ACs covered)
- Graph invariants for REQ/AC/test/doc relationships
- Policy tests (22/22 passing)
- Pre-commit hooks and markdown hygiene
- AC/ADR bidirectional mapping

---

## Verification

```bash
cargo xtask doctor       # Environment validated
cargo xtask selftest     # 8/8 gates pass
cargo xtask ac-status    # 81/81 PASS, 0 FAIL, 0 UNKNOWN
cargo run -p app-http    # Listening on :8080
```

**Detailed AC statuses:** `docs/feature_status.md`

---

## Fork Readiness

The template is ready to fork. Services inheriting from v3.3.3-kernel get:

- Runtime, APIs, and UI that pass their ACs
- DevEx tooling for agents and humans
- Governed workflows with BDD acceptance tests
- Continuous governance validation via selftest
- Agent-friendly documentation and bundler

**Known gaps** (documented in ROADMAP.md):

- Branch protection not configured (manual GitHub setting required - documented in `docs/how-to/setup-branch-protection.md`)
- Tag signing not configured (manual GPG setup required - documented in `docs/how-to/setup-tag-signing.md`)
- Template not yet validated by a second service

**Recently completed (v3.3.3 polish):**

**Documentation:**
- ✅ IDP positioning documentation (`docs/explanation/idp-positioning.md`)
- ✅ Brownfield adoption guide (`docs/guides/brownfield-adoption.md`)
- ✅ Fork feedback workflow (`docs/how-to/report-fork-feedback.md`)
- ✅ Quick start guide (`docs/QUICKSTART.md`)
- ✅ Troubleshooting guide (`docs/TROUBLESHOOTING.md`)
- ✅ Windows development guide (`docs/how-to/windows-development.md`)
- ✅ CI workflows reference (`docs/reference/ci-workflows.md`)
- ✅ Branch protection setup (`docs/how-to/setup-branch-protection.md`)
- ✅ Tag signing setup (`docs/how-to/setup-tag-signing.md`)

**Operational Tooling:**
- ✅ Questions-as-artifacts (`cargo xtask question-new`, `questions-list`)
- ✅ Friction log API (`GET /platform/friction`, `/platform/friction/{id}`)
- ✅ Friction CLI (`cargo xtask friction-new`, `friction-list`)
- ✅ Fork registry (`cargo xtask fork-list`, `fork-register`)
- ✅ Version command (`cargo xtask version` with `--json` support)
- ✅ Release AC deltas in `release-bundle`
- ✅ Branch protection script (`.github/scripts/setup-branch-protection.sh`)

**Fixes:**
- ✅ BDD test isolation (tests no longer pollute tracked files)
- ✅ ADR numbering duplicates resolved

The first real fork will likely discover friction. Capture it in `FRICTION_LOG.md` and consider feeding systematic issues back to the kernel.

---

## End of Kernel Snapshot
