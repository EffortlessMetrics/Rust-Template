---
id: GUIDE-TPL-ROADMAP-001
title: Rust-as-Spec Platform Cell Roadmap
doc_type: guide
status: published
audience: developers, maintainers, platform-engineers
tags: [roadmap, planning, status, milestones]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DOCS-CONSISTENCY]
acs: [AC-PLT-009, AC-PLT-010]
adrs: [ADR-0005]
last_updated: 2026-01-30
---

# Roadmap: Rust-as-Spec Platform Cell (v3.3.14)

This document describes the current state of the template (v3.3.14) and the frozen baseline kernel (v3.3.9-kernel tag).

> For the conceptual model behind Rust-as-Spec, see [`docs/explanation/rust-as-spec-overview.md`](explanation/rust-as-spec-overview.md).

---

## Philosophy: Opinionated Defaults

This template is **not** a generic Rust starter. It encodes specific opinions about governed, agent-friendly development—enforced through Acceptance Criteria in `specs/spec_ledger.yaml`.

### Core Opinions

The template takes strong positions on:

1. **Environment**: Nix-first (Tier-1) with native OS as Tier-2. Reproducibility over convenience.
2. **CI Gate**: `cargo xtask selftest` must pass. Governance validation is non-negotiable.
3. **Governance Artifacts**: Questions, friction, and forks are first-class, queryable data exposed via APIs.
4. **Agent Surfaces**: `/platform/*` endpoints, context bundles, and `--json` CLI outputs for LLM workflows.

Slow is smooth, smooth is fast.

This template is designed to work with long-running agents which work best with well documented consistent interfaces with Schema Gravity.

### Why Opinions Matter

Opinions encoded as ACs mean:

- **Fork customization is explicit**: Change the AC, not random config files
- **Upstream feedback is structured**: Report what opinions didn't fit your domain via `docs/how-to/report-fork-feedback.md`
- **Template evolution is evidence-driven**: Friction from real forks drives AC changes

**To customize:** Identify the AC, modify it, update implementation, verify with selftest. See `docs/QUICKSTART.md` "Defaults & Opinions" section for details.

---

## Roadmap Structure: Three Layers

This roadmap tracks three distinct layers that evolve at different rates:

### Layer 1: Release Train (Template Versions)

Template versions (v3.3.12, v3.3.13, etc.) are **tagged snapshots** of this repository.
`main` may contain post-tag docs/tooling that will ship in the next tag.

| Version | Status | Focus |
| ------- | ------ | ----- |
| v3.3.12 | Tagged | Security hardening, governance architecture, CI improvements |
| v3.3.13 | Tagged | Docs polish + release tooling hardening (see §4.4) |
| v3.3.14 | Tagged | DevEx loop: faster precommit, staged-only semantics (see §4.4.1) |
| v3.3.15 | On main | Security fixes, 12-step selftest, schema registry, release automation (see §4.4.2) |
| v3.4.0 | Planned | External validation: IDP consumer, contract tests, API docs (see §4.5) |
| v3.5.0+ | Vision | Surface minimization: crate extraction (see §4.6) |

### Layer 2: Kernel Baseline (Frozen Tags)

Kernel baseline tags (e.g., `v3.3.9-kernel`) mark frozen API/contract points that forks can pin to. Kernel closures are rare; they happen when contracts stabilize.

| Tag | Status | Notes |
|-----|--------|-------|
| `v3.3.9-kernel` | Frozen | Current stable baseline for forks |
| `v3.4.0-kernel` | Planned | After IDP-ready contract is proven (see §4.5) |

### Layer 3: Adoption Track (Receipts + Forks)

Adoption evidence lives **outside this repo** — in forks, IDP integrations, and agent pilots. Receipts prove the kernel works; friction from real usage drives kernel evolution.

| Evidence              | Location  | Status               |
| --------------------- | --------- | -------------------- |
| Fork dry-run receipt  | Fork repo | Required for v3.4.0  |
| AI first-hour receipt | Fork repo | Required for v3.4.0  |
| IDP tile demo         | IDP repo  | Required for v3.4.0  |

> **Why three layers?** Template patches (v3.3.x) happen frequently. Kernel closures happen rarely. Adoption evidence is captured in other repos. Keeping them separate prevents the roadmap from mixing "what changed on main" with "what's frozen" with "what's proven in real usage."

---

## 1. Kernel v3.3.9 Closure

**Status:** Frozen as of 2025-12-09.

v3.3.9 is the current stable kernel baseline. It includes:

- **Manifest-driven versioning** – `specs/version_manifest.yaml` + versioning engine
- **IDP-ready platform contract** – `/platform/status` and introspection APIs are stable
- **BDD harness exit semantics** (`AC-TPL-BDD-EXIT-CODES`)
- **Agent hint schema** (`AC-TPL-AGENT-HINTS-SCHEMA`)
- **Bundle manifest linkage** (`AC-TPL-BUNDLE-MANIFEST-LINKED`, `AC-TPL-BUNDLE-MINIMAL-SCOPE`)
- **SPEC_ROOT contract** for xtask commands

No new features land here. All forward work targets:

- **Forks**: Build services on v3.3.9 baseline; capture friction
- **v3.4.0**: Kernel improvements driven by real fork feedback

This closure is enforced:

- Git tag: `v3.3.9-kernel` marks the frozen baseline
- `ROADMAP.md` updated to reflect closure
- Selftest gates expanded to 12 steps

To fork from v3.3.9, start with `docs/how-to/FIRST_FORK.md`.

---

## 2. Current State (v3.3.14+ on main)

The template is at v3.3.14 (tagged), with significant improvements on `main` heading toward v3.3.15.

- **Kernel ACs** (`must_have_ac: true`): All passing
- **Selftest**: Green (12/12 gates, expanded from 10)
- **Non-kernel ACs**: Soft gates, may be UNKNOWN depending on test capture

**On `main` (post-v3.3.14):**

Significant work has landed since v3.3.14, including:

- **Security fixes**: DOM-based XSS in AC Coverage view (#146), DoS prevention (#116), auth enforcement on GET/HEAD (#164)
- **12-step selftest**: Docs-as-Code check added as step 2, all subsequent steps renumbered
- **Schema registry compatibility check**: New `cargo xtask schema-check` command (#110)
- **Automated release notes**: Integration with git-cliff for conventional commits (#109)
- **Performance**: Task board optimization (#115), spawn_blocking for UI operations (#142)
- **Microcrate architecture**: gov-http-types, enhanced gov-model with proptest, architecture documentation

**Template Version (v3.3.14 tagged):**

The last released template version includes DevEx improvements (faster precommit, staged-only semantics). Work on main will ship as v3.3.15.

**Frozen Kernel Baseline (v3.3.9-kernel tag):**

The v3.3.9-kernel tag marks the stable, frozen baseline that includes:

- **Manifest-driven versioning engine** – `specs/version_manifest.yaml` declares all version-bearing files; `release-prepare` uses this manifest to update versions atomically
- **Versioning ACs enforced** – `AC-TPL-VERSION-MANIFEST`, `AC-TPL-VERSION-DRYRUN`, `AC-TPL-VERSION-ATOMIC` promoted from deferred to kernel contracts
- **`--dry-run` support** – Preview all version changes before applying them

### 2.1 Governance Status

| Metric | Source |
|--------|--------|
| **Total ACs** | See `docs/feature_status.md` |
| **Kernel ACs** | All passing (`must_have_ac: true`) — see `feature_status.md` for current count |
| **Template ACs** | Soft gates, not enforced — see `feature_status.md` |
| **Meta/CI-only ACs** | CI tests, harness, example tags — see `feature_status.md` |
| **Selftest Gates** | 12/12 passing |
| **Policy Tests** | All passing — run `cargo xtask selftest` for current count |
| **BDD Scenarios** | See `specs/features/` — count tracked in `feature_status.md` |

> **Note:** Selftest only gates on **kernel ACs** (`must_have_ac: true`). Non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking merges. See `docs/feature_status_notes.md` for AC classification details.
>
> **Why no hardcoded counts?** AC/test counts are volatile (depend on which tests ran locally). Only `feature_status.md` (regenerated by `cargo xtask ac-status`) is authoritative. See issue #35.

### 2.2 What's Working

**Runtime & APIs:**

- Service health, version, metrics endpoints
- Platform introspection: `/platform/graph`, `/platform/devex/flows`, `/platform/docs/index`, `/platform/schema`, `/platform/openapi`
- Agent hints API: `/platform/agent/hints` with task prioritization
- Unified issues API: `/platform/issues` aggregating friction, questions, and tasks with pagination
- Platform UI dashboard with graph visualization and flows view
- Configuration validation and IAC alignment (Docker Compose, Kubernetes, Terraform)
- Task lifecycle and governance write operations
- **IDP-ready contract**: The `/platform/*` contract is documented in [`docs/reference/platform_api_contract.md`](reference/platform_api_contract.md) and surfaced in OpenAPI. Reference consumer: [`examples/backstage-plugin/`](../examples/backstage-plugin/)

**DevEx CLI:**

- Development flows: `doctor`, `help-flows`, `check`, `test-changed`, `ac-status`, `ac-coverage`
- Bundler & agent tools: `bundle`, `suggest-next`
- Governance: `adr-new`, `ac-new`, `docs-check`, `graph-export`, `selftest`
- Release management: `release-prepare`, `release-bundle`
- Operational: `audit`, `sbom-local`, `ci-local`, `status`
- Quick validation: `kernel-smoke` (docs-check + selftest)

**Governance:**

- BDD acceptance tests covering all kernel ACs
- Graph invariants for REQ/AC/test/doc relationships
- Policy tests (OPA/Rego) for configuration compliance
- Pre-commit hooks with auto-staging
- AC/ADR bidirectional mapping
- **Docs-as-Code v2:** `spec_ledger.yaml` is the version authority; `docs-check` enforces alignment across consumer files; `xtask version --json` is the machine surface

**v3.3.4 Additions:**

- **BDD Harness**: Semantic exit detection via `is_bdd_success()` - stable across runs
- **Hint Schema**: `/platform/agent/hints` and `xtask suggest-next --format json` share canonical Hint types
- **Bundle Manifests**: `bundle.yaml` links to REQs/ACs/tests with soft scope audit

**v3.3.12 Additions:** See [§3.7](#37-v3312-security--architecture-) for release highlights (security, governance arch, CI).

### 2.3 Verification

```bash
nix develop
cargo xtask doctor         # Environment validated
cargo xtask selftest       # 12/12 gates pass
cargo xtask ac-status      # All kernel ACs PASS
cargo run -p app-http      # Listening on :8080
```

> **Version authority:** `specs/spec_ledger.yaml → metadata.template_version` is the canonical template version. All other docs (README, CLAUDE, ROADMAP, KERNEL_SNAPSHOT, TEMPLATE-CONTRACTS, service_metadata, doc_index, CHANGELOG) are validated against it by `cargo xtask docs-check`.

---

## 3. What's Been Completed (3.3.x Polish)

The following gaps have been addressed:

### 3.1 Documentation ✅

| Item                          | Status     | Location                                 |
| ----------------------------- | ---------- | ---------------------------------------- |
| **IDP positioning doc**       | ✅ Complete | `docs/explanation/idp-positioning.md`    |
| **Brownfield adoption guide** | ✅ Complete | `docs/guides/brownfield-adoption.md`     |
| **Fork feedback workflow**    | ✅ Complete | `docs/how-to/report-fork-feedback.md`    |
| **Quick start guide**         | ✅ Complete | `docs/QUICKSTART.md`                     |
| **Troubleshooting guide**     | ✅ Complete | `docs/TROUBLESHOOTING.md`                |
| **Windows development guide** | ✅ Complete | `docs/how-to/windows-development.md`     |
| **CI workflows reference**    | ✅ Complete | `docs/reference/ci-workflows.md`         |
| **Branch protection setup**   | ✅ Complete | `docs/how-to/setup-branch-protection.md` |
| **Tag signing setup**         | ✅ Complete | `docs/how-to/setup-tag-signing.md`       |
| **Environment setup guide**   | ✅ Complete | `docs/reference/environment.md`          |

### 3.2 Operational Tooling ✅

| Item                         | Status     | Command/Endpoint                                                             |
| ---------------------------- | ---------- | ---------------------------------------------------------------------------- |
| **Questions-as-artifacts**   | ✅ Complete | `cargo xtask question-new`, `questions-list`                                 |
| **Friction log API**         | ✅ Complete | `GET /platform/friction`, `/platform/friction/{id}`                          |
| **Fork registry**            | ✅ Complete | `cargo xtask fork-list`, `fork-register`                                     |
| **Version command**          | ✅ Complete | `cargo xtask version` (with `--json`)                                        |
| **Friction CLI**             | ✅ Complete | `cargo xtask friction-new`, `friction-list`                                  |
| **Release AC deltas**        | ✅ Complete | `cargo xtask release-bundle` now includes AC changes                         |
| **Branch protection script** | ✅ Complete | `.github/scripts/setup-branch-protection.sh`                                 |
| **Service-init command**     | ✅ Complete | `cargo xtask service-init` - single command fork branding                    |
| **JSON CLI outputs**         | ✅ Complete | `--json` flag on `ac-status`, `friction-list`, `questions-list`, `fork-list` |
| **Questions HTTP API**       | ✅ Complete | `GET /platform/questions`, `/platform/questions/{id}`                        |
| **Forks HTTP API**           | ✅ Complete | `GET /platform/forks`, `/platform/forks/{name}`                              |
| **Unified issues endpoint**  | ✅ Complete | `GET /platform/issues` with filtering, pagination, cross-artifact search (PR #74) |
| **Issues CLI search**        | ✅ Complete | `cargo xtask issues-search` unified search across friction, questions, tasks |

### 3.3 Editor Integration ✅

| Item                       | Status     | Location                  |
| -------------------------- | ---------- | ------------------------- |
| **VS Code extensions**     | ✅ Complete | `.vscode/extensions.json` |
| **VS Code tasks**          | ✅ Complete | `.vscode/tasks.json`      |
| **VS Code launch configs** | ✅ Complete | `.vscode/launch.json`     |
| **VS Code settings**       | ✅ Complete | `.vscode/settings.json`   |

### 3.4 Test Isolation ✅

| Item                   | Status  | Notes                                 |
| ---------------------- | ------- | ------------------------------------- |
| **BDD test isolation** | ✅ Fixed | Tests no longer pollute tracked files |

### 3.5 Technical Debt ✅

| Item                            | Status  | Notes                                        |
| ------------------------------- | ------- | -------------------------------------------- |
| **ADR numbering duplicates**    | ✅ Fixed | Removed test scaffolds, renumbered 0007→0019 |
| **Release evidence incomplete** | ✅ Fixed | AC delta tracking added to release-bundle    |

### 3.6 v3.3.4 Polish (Harness, Hints, Bundles) ✅

| Item                      | Status     | Notes                                                                      |
| ------------------------- | ---------- | -------------------------------------------------------------------------- |
| **BDD harness semantics** | ✅ Complete | `bdd.rs` + `is_bdd_success()` (`AC-TPL-BDD-EXIT-CODES`)                    |
| **Agent Hint schema**     | ✅ Complete | `AC-TPL-AGENT-HINTS-SCHEMA` + BDD + unit tests                             |
| **Bundle manifest v1.5**  | ✅ Complete | `AC-TPL-BUNDLE-MANIFEST-LINKED` + `AC-TPL-BUNDLE-MINIMAL-SCOPE`            |
| **SPEC_ROOT contract**    | ✅ Complete | `AC-TPL-XTASK-SPEC-ROOT` — `spec_root()` honors env var                    |
| **Pre-commit auto-fix**   | ✅ Complete | Hook runs `xtask precommit`, auto-fixes fmt/skills/feature_status          |
| **Docs-as-Code v2**       | ✅ Complete | `AC-PLT-009` + `AC-PLT-010` — see `docs/explanation/TEMPLATE-CONTRACTS.md` |

### 3.7 v3.3.12 Security & Architecture ✅

#### Security Hardening

| Item                        | Status     | Notes                                                                |
| --------------------------- | ---------- | -------------------------------------------------------------------- |
| **Security headers**        | ✅ Complete | CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Permissions-Policy |
| **CORS middleware**         | ✅ Complete | Configurable origins, methods, headers with secure defaults          |
| **Enhanced JWT validation** | ✅ Complete | 60s leeway, claim validation (iss, sub, iat), expiration enforcement |
| **Fail-closed auth**        | ✅ Complete | Invalid auth modes fail loudly instead of falling back to "open"     |
| **Supply chain CI**         | ✅ Complete | CodeQL, Gitleaks, cargo-audit, cargo-deny, checksum enforcement      |

#### Governance Architecture Refactoring

| Item                         | Status     | Notes                                                               |
| ---------------------------- | ---------- | ------------------------------------------------------------------- |
| **gov-model crate**          | ✅ Complete | Pure domain types: Task, TaskStatus, GovernanceRepository trait     |
| **gov-http crate**           | ✅ Complete | Reusable Axum router for all `/platform/*` endpoints                |
| **PlatformState trait**      | ✅ Complete | Dependency injection abstraction for HTTP handlers                  |
| **RepoContext centralized**  | ✅ Complete | Workspace path resolution unified across kernel crates              |
| **Handler modularization**   | ✅ Complete | friction, questions, forks as composable router submodules          |

#### CI Improvements

| Item                         | Status     | Notes                                                               |
| ---------------------------- | ---------- | ------------------------------------------------------------------- |
| **Three-tier path filtering**| ✅ Complete | docs-check / check / selftest tiers based on changed files          |
| **Shared rust-cache**        | ✅ Complete | Composite actions for consistent caching across workflows           |
| **Release readiness guide**  | ✅ Complete | `docs/how-to/release-readiness-checklist.md`                        |

---

## 4. What's Still Needed

Only a few items remain - all now have documentation or are external dependencies:

### 4.1 Manual Configuration Required

| Item                  | Impact                                  | Documentation                            | Action Required                                  |
| --------------------- | --------------------------------------- | ---------------------------------------- | ------------------------------------------------ |
| **Branch protection** | CI can be bypassed                      | `docs/how-to/setup-branch-protection.md` | Run `.github/scripts/setup-branch-protection.sh` |
| **Tag signing**       | Releases not cryptographically verified | `docs/how-to/setup-tag-signing.md`       | Set up GPG key and configure Git                 |

### 4.2 External Validation (Out of Scope for This Repo)

| Gap                             | Impact                                       | Effort | Notes                                                                         |
| ------------------------------- | -------------------------------------------- | ------ | ----------------------------------------------------------------------------- |
| **No second service built yet** | Template assumptions untested in a real fork | High   | Requires building a service in another repo using this template as a baseline |

> This is intentionally **outside the scope of this repository**. This roadmap tracks it as a recommended next step for adopters, not as work to be done here. The kernel is complete; validation happens in forks.

### 4.3 External Dependencies

| Item                         | Impact                   | Notes                                                                                                                                 |
| ---------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| **`lazy-trees` Nix warning** | Cosmetic noise in output | Deprecated Nix 2.30+ setting in Determinate installer config. Documented in TROUBLESHOOTING.md with fix instructions. Safe to ignore. |

### 4.4 v3.3.13 – Docs Polish & Release Tooling (Patch Release)

> **Scope:** Documentation alignment and release tooling hardening. The platform work (security,
> architecture) is complete. Adoption receipts are tracked in v3.4.0 entry criteria, not as
> v3.3.13 blockers.

**Status:** Released (tag: v3.3.13)

#### Shipped in v3.3.13

| Item                         | Description                                   | Status              |
| ---------------------------- | --------------------------------------------- | ------------------- |
| **Docs version alignment** | All docs updated to v3.3.13 references | ✅ Merged (PR #40) |
| **Security configuration doc** | Auth modes, CORS, JWT, headers, fail-closed | ✅ Merged (PR #38) |
| **Selftest green** | 12/12 gates passing | ✅ Verified |

#### Already Complete (From v3.3.12)

The following platform work landed in v3.3.12 and is **not** v3.3.13 scope.
See [§3.7](#37-v3312-security--architecture-) for details:

- Security middleware (CORS, headers, JWT validation, fail-closed auth) — PR #33
- Governance architecture (gov-model, gov-http, handler modularization) — Refactoring
- Supply chain CI (CodeQL, Gitleaks, cargo-audit) — PR #33
- Documentation templates (Trust a Cell, Evolve the Kernel, AI first-hour receipt template)

#### Release Checklist

```text
[x] Add evidence bundle: cargo xtask release-bundle 3.3.13
[x] Commit evidence and merge to main
[x] Tag: git tag v3.3.13 -m "v3.3.13"
[x] Push: git push --follow-tags
```

#### Adoption Track (v3.4.0 Entry Criteria)

These receipts validate the template in real use but are **not** v3.3.13 blockers:

| Item | Description | Status | Notes |
| ---- | ----------- | ------ | ----- |
| **Fork dry-run receipt** | Real fork from `v3.3.9-kernel`, full ladder green | 🔜 Pending | Required before v3.4.0 work |
| **AI first-hour receipt** | Agent run through `ai-first-hour.md` | 🔜 Pending | Required before v3.4.0 work |

#### Deferred (Known Issues, Not Blockers)

| Item                 | Description                                     | Notes            |
| -------------------- | ----------------------------------------------- | ---------------- |
| **IDP adapter stub** | Backstage/Port adapter consuming `idp-snapshot` | v3.4.0 territory |

#### Recently Resolved

| Item                       | Description                                 | Resolution              |
| -------------------------- | ------------------------------------------- | ----------------------- |
| **sccache/libz friction**  | FRICTION-ENV-001 affecting `nix develop -c` | ✅ Fixed in v3.3.12     |
| **Docs version drift**     | Multiple docs referenced v3.3.12            | ✅ Fixed, merged (PR #40) |

**Definition of Done for v3.3.13:**

1. Security config doc merged — ✅ complete (PR #38)
2. Docs version alignment merged — ✅ complete (PR #40)
3. `cargo xtask selftest` green — ✅ verified
4. Evidence bundle committed — ✅ complete (PR #41)
5. Tag pushed — ✅ complete (v3.3.13)

> **Status:** v3.3.13 is **released**. Fork and AI receipts are v3.4.0 entry criteria.

---

### 4.4.1 v3.3.14 – DevEx Loop (Patch Release)

> **Scope:** Developer experience improvements and platform enhancements merged after v3.3.13.
> Includes DevEx loop improvements (PRs #43-45), OpenAPI endpoint (PR #61), unified issues management (PRs #74-76).

**Status:** Released (tag: v3.3.14)

#### Merged (Post-v3.3.13)

| Item | Description | Status |
| ---- | ----------- | ------ |
| **Faster precommit** | Default mode changed from full to fast; ~10x speedup for typical commits | ✅ Merged (PR #43) |
| **Staged-only semantics** | `--staged-only` flag limits checks to staged files only | ✅ Merged (PR #43) |
| **Targeted spellcheck** | Spellcheck runs only on changed `.md` files in fast mode | ✅ Merged (PR #43) |
| **docs-check alignment** | Consistent behavior between precommit and CI | ✅ Merged (PR #43) |
| **Blocking hook** | Pre-commit hook now blocks on failure (escape hatch: `--no-verify`) | ✅ Merged (PR #45) |
| **Staged-only Rust policy** | Requires clean Rust worktree when staging Rust changes | ✅ Merged (PR #45) |
| **Worktree cleanup** | Acceptance tests clean up git worktrees properly (fixes ENOENT spam) | ✅ Merged (PR #45) |
| **OpenAPI endpoint** | `/platform/openapi` returns OpenAPI 3.0 spec for platform APIs | ✅ Merged (PR #61) |
| **Unified issues endpoint** | `/platform/issues` aggregates friction, questions, tasks with pagination | ✅ Merged (PR #74) |
| **Issues CLI search** | `cargo xtask issues-search` for cross-artifact unified search | ✅ Merged (PR #74) |
| **xtask/gov-http hardening** | Stabilized git calls, improved error handling after issues merge | ✅ Merged (PR #75) |
| **Pagination BDD scenarios** | BDD coverage for pagination error contract (400 responses) | ✅ Merged (PR #76) |

#### Hook Behavior (Design Decision)

The pre-commit hook is **blocking by default**. This ensures bad commits don't slip through:

- **Hook blocks on failure** — if checks fail, the commit is aborted
- **Fast mode runs in hook** — quick format/lint/clippy, not full selftest
- **Auto-fix what's mechanical** — fmt and skill/agent formatting auto-stage changes
- **Escape hatch available** — `git commit --no-verify` bypasses when truly needed
- **Full mode for receipts** — `cargo xtask precommit --mode full` runs CI-grade checks

**Staged-only Rust policy:** When staging Rust changes with `--staged-only`, the worktree must be clean for Rust-affecting files. This prevents fmt from auto-fixing unstaged files or clippy from failing on WIP code. If you have unstaged Rust changes, the hook will error with clear remediation steps.

To customize hook strictness:

- `cargo xtask precommit --mode full` — runs receipt-grade checks (same as CI)
- `XTASK_STRICT_PRECOMMIT=1` — makes docs-check and spellcheck hard-fail instead of warn

#### Release Checklist

```text
[x] Verify selftest green: cargo xtask selftest
[x] Add evidence bundle: cargo xtask release-bundle 3.3.14
[x] Commit evidence and merge to main
[x] Tag: git tag v3.3.14 -m "v3.3.14"
[x] Push: git push --follow-tags
```

#### Optional Follow-ups (Not Blocking Tag)

| Item | Description | Notes |
| ---- | ----------- | ----- |
| **pre-push hook option** | Install hooks could optionally add a pre-push hook for full validation | Keeps commits fast, pushes safe |
| **Untracked file detection** | Include `git ls-files --others` in non-staged mode | Catches new files that aren't staged |
| **Spellcheck specs YAML** | Extend spellcheck targets to include `specs/*.yaml` | Currently markdown-only |

---

### 4.4.2 Post-v3.3.14 – v3.3.15 Scope (Security, Performance, Tooling)

> **Scope:** Security fixes, performance improvements, and tooling enhancements merged after v3.3.14 tag.
> This will ship as v3.3.15.

**Status:** In Progress on `main` (30+ commits since v3.3.14)

#### Security Fixes

| Item | Description | Status |
| ---- | ----------- | ------ |
| **DOM-based XSS fix** | Fixed high-severity XSS vulnerability in AC Coverage HTML view (PR #146) | ✅ Merged |
| **DoS prevention** | Fixed potential denial-of-service in constant-time auth comparison (PR #116) | ✅ Merged |
| **Auth enforcement** | GET/HEAD requests now require auth when credentials configured (PR #164) | ✅ Merged |
| **Checksum enforcement** | Database tooling downloads verified with checksums in CI (PR #111) | ✅ Merged |

#### Selftest & Governance

| Item | Description | Status |
| ---- | ----------- | ------ |
| **12-step selftest** | Docs-as-Code check added as step 2; all steps renumbered (PR #159) | ✅ Merged |
| **AC status updates** | Multiple requirements updated from UNKNOWN to PASS status | ✅ Merged |
| **Kernel req coverage** | Clarified error messages for kernel requirement doc coverage (PR #114) | ✅ Merged |

#### Tooling & Automation

| Item | Description | Status |
| ---- | ----------- | ------ |
| **Schema registry check** | New `cargo xtask schema-check` validates schema compatibility (PR #110) | ✅ Merged |
| **Automated release notes** | Integration with git-cliff for conventional commit changelogs (PR #109) | ✅ Merged |
| **Scanner-safe test secrets** | Fake secrets in tests replaced with EXAMPLE_* patterns | ✅ Merged |

#### Performance & Architecture

| Item | Description | Status |
| ---- | ----------- | ------ |
| **Task board optimization** | Optimized HTML generation for task board view (PR #115) | ✅ Merged |
| **UI spawn_blocking** | Blocking UI operations offloaded to spawn_blocking (PR #142) | ✅ Merged |
| **UI accessibility** | Added aria-current to active navigation links (PR #162) | ✅ Merged |
| **Test isolation** | Eliminated unsafe env/cwd mutation via guarded utilities (PR #106) | ✅ Merged |
| **Async fixes** | tokio::sync::RwLock in adapters-grpc async tests (PR #103) | ✅ Merged |

#### Microcrate Architecture Polish

| Item | Description | Status |
| ---- | ----------- | ------ |
| **New `gov-http-types` crate** | Shared HTTP API types extracted to reduce duplication | ✅ Merged |
| **Enhanced `gov-model`** | TaskStatus parsing with aliases and property-based tests (proptest) | ✅ Merged |
| **Ignored tests documentation** | `docs/reference/ignored-tests.md` catalogs all `#[ignore]` tests | ✅ Merged |
| **Architecture doc update** | Complete crate taxonomy (21+ crates across 6 layers) documented | ✅ Merged |
| **Crate dependency cleanup** | gov-http-friction/questions now use shared types from gov-http-types | ✅ Merged |

#### Architecture Highlights

The microcrate architecture now clearly separates crates into 6 layers:

1. **Contract** — Stable APIs (`platform-contract`, `gov-contracts`, `receipts-core`)
2. **Core Logic** — Business rules (`gov-model`, `gov-policy`, `spec-ledger`)
3. **Foundation** — Shared utilities (`http-core`, `gov-http-types`, `telemetry`)
4. **Adapter** — External interfaces (`gov-http-*`, `adapters-db-sqlx`)
5. **HTTP/Router** — Application entry (`app-http`, `http-middleware`)
6. **Facade** — Developer tooling (`gov-xtask-core`, `rust_iac_config`)

This layering enables future crate extraction (v3.5.0+ vision) by establishing clear dependency boundaries now.

#### Release Checklist for v3.3.15

```text
[ ] Verify selftest green: cargo xtask selftest
[ ] Update version in spec_ledger.yaml and related files
[ ] Add evidence bundle: cargo xtask release-bundle 3.3.15
[ ] Commit evidence and merge to main
[ ] Tag: git tag v3.3.15 -m "v3.3.15"
[ ] Push: git push --follow-tags
```

---

### 4.5 v3.4.0 – External Validation (Minor Release)

> **Note:** v3.4.0 is the *next minor* kernel closure. The current frozen baseline is `v3.3.9-kernel`.

#### What v3.4.0 Proves

v3.4.0 is explicitly the **"external proof" release**. It demonstrates that the template works beyond selftest:

| Criterion | Definition of Done |
| --------- | ------------------ |
| **One reference consumer** | A real IDP tile (Backstage or similar) consuming `/platform/*` endpoints |
| **Contract tests for `/platform/*`** | Schema-level regression detection for API stability |
| **Multi-service registry spec** | Even static YAML for listing cells + one aggregator query (fleet view) |
| **Curl-first API examples** | 2-3 examples per endpoint (happy path + auth failure + schema link) |
| **Receipts validate adoption** | Fork dry-run + AI first-hour receipts captured in real forks |

The gap v3.4.0 closes: **"tests are green" vs "ship survives real load"**.

#### Entry Criteria (Gate Before Starting v3.4.0)

- v3.3.12 released ✅
- v3.3.13 released ✅
- v3.3.14 released ✅
- Fork dry-run receipt 🔜 pending
- AI first-hour receipt 🔜 pending
- At least one real fork exists and is actively used
- Friction log reviewed; v3.4.0 candidates tagged

#### Planned Work (Demand-Driven)

| Item | Description | Status | Priority |
| ---- | ----------- | ------ | -------- |
| **Reference IDP consumer** | Real Backstage tile consuming `/platform/status` + `/platform/docs/index` | 🔜 Planned | High |
| **Contract tests** | OpenAPI schema validation tests for `/platform/*` endpoints | 🔜 Planned | High |
| **Multi-service registry spec** | Static YAML registry listing cells + `idp-snapshot` endpoints | 🔜 Planned | High |
| **API docs curl examples** | Expand `docs/api/README.md` with curl-first examples per endpoint | 🔜 Planned | Medium |
| **Friction taxonomy + promotion** | Workflow for soft → hard gate promotion based on fork feedback | 🔜 Planned | Medium |

#### Deferred to v3.5.0+

These belong in later releases unless adoption pressure forces them earlier:

| Item | Description | Rationale |
| ---- | ----------- | --------- |
| **Crate extraction** | Publish `gov-model`, `gov-http` as standalone crates | See §4.6 Surface Minimization |
| **Cross-cell graph queries** | Query governance state across multiple cells | Needs registry + real multi-cell usage |
| **Advanced policy packs** | PCI-DSS, HIPAA compliance templates | Domain-specific; not core |
| **Fleet-wide Backstage integration** | Plugin reading `/platform/*` from N services | v3.5.0 after registry is proven |

---

### 4.6 v3.5.0+ – Surface Minimization (Future)

> **Goal:** Reduce fork burden by extracting reusable machinery into published crates.

#### The Problem

Right now "template" implies copying lots of repo machinery forward. Forks drag half the factory with them, making upgrades painful and creating forever-forks.

#### The Solution

Separate **reusable crates** from the **template skeleton**:

| Crate | Purpose | Current Location |
| ----- | ------- | ---------------- |
| `gov-model` | Pure domain types: Task, TaskStatus, GovernanceRepository trait | `crates/gov-model/` |
| `gov-http` | Reusable Axum router for `/platform/*` endpoints | `crates/gov-http/` |
| `ac-kernel` | AC/test mapping, spec_ledger parsing, BDD harness | `crates/ac-kernel/` |
| `versioning` | Manifest-driven version engine | Parts of `crates/xtask/` |

#### End State

- **Published crates:** Adopters depend on crates via Cargo, not vendoring
- **Thin template:** Focused on composition + governance + domain examples
- **Easier upgrades:** Bump crate version, not merge entire repo

#### Entry Criteria

- v3.4.0 shipped with real IDP consumer
- At least 2 active forks experiencing upgrade friction
- Clear boundary between "kernel machinery" and "template examples"

#### Not Yet Planned

This is a **vision**, not a commitment. Work here is demand-driven by real fork friction, not speculative.

#### Already in v3.3.9-kernel

Items completed and part of the frozen kernel:

| Item                          | Description                                                            | Status      |
| ----------------------------- | ---------------------------------------------------------------------- | ----------- |
| **IDP Snapshot Contract**     | `cargo xtask idp-snapshot` + `/platform/idp/snapshot` emit stable JSON | ✅ In v3.3.7 |
| **Platform Schema**           | `specs/platform_schema.yaml` defines all endpoint responses            | ✅ In v3.3.7 |
| **Docs-as-Code v3**           | Bidirectional `doc_index.yaml` ↔ front-matter sync                     | ✅ In v3.3.7 |
| **Feature-status invariants** | `feature_status.md` header matches spec_ledger version                 | ✅ In v3.3.7 |
| **Centralized Env Helpers**   | `crate::env::is_ci()`, `is_noninteractive()` reduce duplication        | ✅ In v3.3.7 |
| **Example fork CI**           | `examples/fork-customization/` with CI template                        | ✅ In v3.3.7 |

See [v3.4.0-plan.md](plans/v3.4.0-plan.md) for scope when v3.4.0 work begins.

---

### 4.7 Publishing & Forensics Track

> **Goal:** Make PR archaeology operational so it yields better cover sheets, an auditable casebook, and concrete factory improvements.

This track runs in parallel with feature work. It builds the "truth surface" that makes large AI-assisted changes trustworthy.

#### P0 — Audit Pack Docs (Foundation) ✅

**Status:** Complete

**Done when:**
- [x] `docs/audit/AUDIT_PATH.md` exists (15-minute verification guide)
- [x] `docs/audit/PROVENANCE.md` exists (trust model, automation vs human)
- [x] `docs/audit/PR_COVER_SHEET.md` exists (canonical format)
- [x] `docs/audit/RECEIPTS.md` exists (schemas documented)
- [x] `docs/audit/FAILURE_MODES.md` exists (taxonomy)
- [x] `docs/audit/CASEBOOK.md` exists (curated exhibits)

#### P1 — Receipt Schemas (Machine Surface)

**Status:** Planned

**Done when:**
- [ ] `gate.json` schema is stable (what ran, pass/fail, versions, timestamps)
- [ ] `economics.json` schema is stable (DevLT + compute, allows unknowns)
- [ ] `dossier.json` schema is stable (scope, findings, exhibit score)
- [ ] Schemas documented in `docs/audit/RECEIPTS.md`

#### P2 — PR Cover Sheet Generator

**Status:** Planned

**Done when:**
- [ ] `xtask pr-cover --pr <n> --run-dir <path>` outputs markdown block
- [ ] Block includes: review map + proof links + known limits + errata + reproduce
- [ ] Generator is deterministic (same inputs → same output)
- [ ] Claim updates derived from receipts only

#### P3 — PR Updater (Safe Write)

**Status:** Planned

**Done when:**
- [ ] `xtask pr-update --pr <n> --run-id <id>` replaces only bounded block
- [ ] Never touches content outside the cover sheet section
- [ ] Writes version-controlled copy to `docs/audit/EXHIBITS/PR-<n>.md`

#### P4 — Dossier + Casebook Generator

**Status:** Planned

**Done when:**
- [ ] `xtask pr-dossier --pr <n>` produces structured dossier JSON
- [ ] `xtask casebook-gen` builds/updates `docs/audit/CASEBOOK.md` from dossiers
- [ ] Casebook includes DevLT/compute fields and "what went wrong" sections

#### P5 — Factory Backlog Extractor

**Status:** Planned

**Done when:**
- [ ] Recurring failure modes counted (by taxonomy)
- [ ] Top offenders generate issues/friction entries
- [ ] Backlog ties back to specific PRs/dossiers

#### Optional Future

- P6 — GitHub "Swarm Gate" check-run posting (if GitHub UI needed)
- P7 — Cross-repo exhibit aggregation (fleet-wide casebook)

---

## 5. Path Forward Options

> **Current State:** Template v3.3.14 is released. The next step is external validation via fork receipts (v3.4.0 entry criteria).

### 5.1 Option A: Minimal (Lock and Fork) — *Active*

**Goal:** Freeze the kernel as-is, use it for services, let friction drive improvements.

**Status:** This is the current path. v3.3.9-kernel is frozen; v3.3.14 is released; v3.3.15 is in progress on main.

**Immediate Next Steps:**

1. ✅ Kernel frozen at v3.3.9-kernel
2. ✅ Branch protection configured
3. ✅ Documentation complete (v3.3.12 -> v3.3.13)
4. ✅ v3.3.13 tagged with evidence bundle
5. ✅ v3.3.14 tagged (DevEx improvements + unified issues endpoint)
6. 🔜 Tag v3.3.15 (security fixes + 12-step selftest + tooling)
7. 🔜 Create fork from v3.3.9-kernel tag
8. 🔜 Complete fork dry-run and AI first-hour receipts (v3.4.0 gate)
9. 🔜 v3.4.0: External validation (reference consumer, contract tests, API examples)

**After v3.3.14:**

- Fork for real service development
- Capture friction in `FRICTION_LOG.md`
- Report kernel issues via `docs/how-to/report-fork-feedback.md`
- Kernel improvements batch into v3.4.0 based on real friction

### 5.2 Option B: Consolidate (Fill Documentation Gaps) — *Complete*

> **Note:** Option B was executed during v3.3.3 → v3.3.12. All items complete.

**Completed Items:**

1. ✅ `docs/explanation/idp-positioning.md`
2. ✅ `docs/how-to/brownfield-adoption.md`
3. ✅ Branch protection setup documented
4. ✅ Windows Tier-2 flow documented
5. ✅ ADR numbering cleaned up
6. ✅ Security configuration doc (v3.3.13)
7. ✅ Version alignment across all docs (v3.3.13)

**Result:** Documentation is comprehensive. Onboarding friction should be minimal.

### 5.3 Option C: Validate (Second Service First) — **Adoption Track, Not Kernel Work**

> This option describes how to use the template in a **separate service repository**.
> It does not require changes to this repository once `v3.3.9-kernel` is tagged.

**Goal:** Prove the template works by building a real service from it.

**Actions (in the fork repo, not here):**

1. Fork for Knowledge Hub or other service
2. Document friction as it occurs
3. Report kernel issues back via `docs/how-to/report-fork-feedback.md`
4. Extract patterns into fork docs after validation
5. Kernel improvements (if any) land in v3.4.0 based on real friction

**Timeline:** 2-4 sessions (in the fork)

**Pros:**

- Documentation grounded in reality
- Discovers actual problems
- Template improvements earned, not assumed

**Cons:**

- Friction during development
- May need kernel changes mid-service
- More churn before stability

---

## 5A. Adoption Phases (Build-Out Track)

> **Scope:** These phases happen **in forks and platform infrastructure**, not in this kernel repo. Kernel improvements are batched at phase gates based on demand, not speculation.

### Phase 1: Fork Usage Baseline

**Goal:** Test kernel invariants in real service environment

**Activities:**

- Fork template into new service using `v3.3.9-kernel` tag
- Run `nix develop && cargo xtask doctor && cargo xtask selftest` to validate baseline
- Wire in service identity via `specs/service_metadata.yaml`
- Add domain stories/REQs/ACs to `specs/spec_ledger.yaml`
- Capture friction in fork's `FRICTION_LOG.md` and surface via `/platform/friction` API
- Use `docs/how-to/report-fork-feedback.md` to report kernel issues upstream

**Success Criteria:**

- Selftest green on every PR in fork repo
- Friction systematically logged and categorized
- Service builds and runs with template governance intact

### Phase 2: IDP Tile Integration

**Goal:** Surface governance and docs health in developer portal

**Activities:**

- Build **Governance Health tile** using `/platform/status` endpoint

  - Show AC pass/fail counts, policy status, selftest gate results
- Build **Docs Health tile** using `/platform/docs/index` endpoint

  - Show doc types, coverage, staleness, missing entries
- (Optional) Add **Task/Hints tile** using `/platform/agent/hints` endpoint

  - Surface prioritized work items for teams and agents
- Reference `docs/explanation/json-contracts.md` for JSON schema contracts
- Validate tile data against fork services (Phase 1 outputs)

**Success Criteria:**

- Template-based services visible in IDP with health metrics
- Teams can see governance drift in real-time
- Documentation health surfaced without manual audits

### Phase 3: Governed Agent Pilot

**Goal:** Validate kernel is truly agent-friendly

**Activities:**

- Deploy Claude Code agents to 2-3 fork repos from Phase 1
- Use Skills: `bootstrap-dev-env`, `governed-feature-dev`, `governed-maintenance`
- Agent workflow loop:

  1. Query `/platform/agent/hints` for prioritized tasks
  2. Generate context bundle via `cargo xtask bundle <task_name>`
  3. Edit code/tests/docs within bundle scope
  4. Validate with `cargo xtask test-ac <AC_ID>`
  5. Gate on `cargo xtask selftest` before PR
- Require AC/REQ/Doc invariants green (`docs-check` + `selftest`)
- Capture agent friction separately from human developer friction

**Success Criteria:**

- Agents productive in 2-3 real service repos without human intervention
- Agent-generated PRs pass selftest on first attempt >80% of the time
- Clear friction log distinguishing agent vs. human developer pain points

### Phase 4: Kernel vNext (Demand-Driven)

**Goal:** Batch real feedback into next kernel version

**Activities:**

- Review friction logs from Phases 1-3 (fork usage + IDP + agents)
- Categorize feedback:

  - **Kernel fixes:** gaps in `spec_ledger.yaml`, broken contracts, missing flows
  - **Soft → hard promotions:** checks validated in real usage, ready to gate
  - **JSON contract refinements:** IDP/agent usage reveals schema gaps
  - **Out-of-scope:** fork-specific needs, not generalizable
- Promote soft checks to hard gates after validation (e.g., `docs-check` strictness)
- Refine JSON contracts based on IDP tile and agent integration patterns
- Implement versioning engine refactor if `release-prepare` friction is systematic
- Add new patterns discovered in forks (e.g., common service types, IAC extensions)

**Success Criteria:**

- v3.4.0 (or v4.0.0) released with changes **driven by friction**, not speculation
- All promoted hard gates have evidence from ≥2 fork repos
- JSON contracts validated by real IDP/agent consumers
- Kernel changelog clearly attributes improvements to fork feedback

---

**Note:** These phases are **adoption-driven** and happen outside this kernel repository. Kernel improvements are batched at phase gates (especially Phase 4) rather than landed speculatively. The v3.3.9-kernel is complete; validation and evolution happen through real usage.

---

## 6. Recommended Path (For Adopters)

> **Scope note:** This section describes what happens **in forks**, not in this repo.
> The kernel (v3.3.9) is frozen. Template is at v3.3.14. Validation happens when you use it.

**Option C (Validate) is recommended** for these reasons:

1. **100% AC pass doesn't mean ready.** The ACs test what we said we'd build, not what services actually need.
2. **Documentation written after use is better.** We'll know what to document because we'll have hit the gaps.
3. **Friction is valuable signal.** The first fork will generate a friction log that tells us exactly what to fix.

### Getting Started (For New Adopters)

```bash
# 1. Fork from the kernel baseline
git clone --branch v3.3.9-kernel <your-fork-url>
cd your-service

# 2. Validate the baseline works
nix develop
cargo xtask dev-up          # One-command setup + validation

# 3. Brand your service
cargo xtask service-init \
  --id my-service \
  --name "My Service" \
  --description "Service description"

# 4. Start developing
cargo xtask help-flows      # See available workflows
```

### Recommended Sequence (In Your Fork)

```text
Day 1: Fork and validate
       - Fork from v3.3.9-kernel tag
       - Run cargo xtask dev-up (must pass)
       - Run cargo xtask selftest (must be green)

Days 2-5: Service development
       - Add domain REQs/ACs to specs/spec_ledger.yaml
       - Use bundler: cargo xtask bundle implement_ac
       - Use agent hints: curl localhost:8080/platform/agent/hints
       - Capture friction immediately in FRICTION_LOG.md

Week 2: First feature complete
       - AC-first development workflow validated
       - Domain BDD scenarios in specs/features/
       - cargo xtask selftest green with domain ACs

Week 3+: Kernel feedback
       - Review friction log
       - Categorize: kernel fix vs. service-specific vs. doc-only
       - Report kernel issues via docs/how-to/report-fork-feedback.md
       - Kernel fixes batch into v3.4.0
```

---

## 7. Quick Reference

### 7.1 Day-0 Commands (New Clone)

```bash
nix develop                    # Enter devshell
cargo xtask doctor             # Verify environment
cargo xtask kernel-smoke       # Quick validation
cargo run -p app-http          # Start service
```

### 7.2 Fork Initialization

```bash
cargo xtask service-init \
  --id my-service \
  --name "My Service" \
  --description "My service description"
```

### 7.3 Development Commands

```bash
cargo xtask check              # Fast local checks
cargo xtask test-changed       # Run affected tests
cargo xtask test-ac AC-XXX     # Test specific AC
cargo xtask ac-status          # View AC coverage (--json for machine output)
cargo xtask selftest           # Full governance check
cargo xtask version            # Show kernel version (--json for machine output)
```

### 7.4 Governance Artifacts

```bash
cargo xtask friction-new --category X --severity Y --summary "..."
cargo xtask friction-list      # List friction entries (--json available)
cargo xtask question-new --category X --summary "..." --flow F --phase P --description "..."
cargo xtask questions-list     # List questions (--json available)
cargo xtask fork-register --name "Name" --domain "domain" --kernel-version "vX.Y.Z" ...
cargo xtask fork-list          # List registered forks (--json available)
```

### 7.5 Agent Workflow

```bash
cargo xtask bundle implement_ac    # Get context bundle
cargo xtask suggest-next           # Get task suggestions
cargo xtask issues-search "query"  # Search across all artifact types
# Platform APIs for agents
curl localhost:8080/platform/agent/hints   # Prioritized task hints
curl localhost:8080/platform/questions     # Question artifacts
curl localhost:8080/platform/forks         # Fork registry
curl localhost:8080/platform/friction      # Friction log
curl localhost:8080/platform/issues        # Unified issues (friction + questions + tasks)
```

### 7.6 Release Commands

```bash
cargo xtask release-prepare X.Y.Z
cargo xtask selftest
cargo xtask release-bundle X.Y.Z
```

---

## 8. Definition of Done

### 8.1 Kernel Definition of Done (This Repo)

The template kernel is "done" when:

1. **All kernel ACs pass** (`must_have_ac: true` in `spec_ledger.yaml`)
2. **`cargo xtask selftest` is green** on Tier-1 (Nix devshell) and enforced in CI
3. **Docs-as-Code invariants hold:**

   - Version alignment (`AC-PLT-009`, `AC-PLT-010`, extended with feature-status invariants)
   - `doc_index.yaml` ↔ front-matter sync (`AC-PLT-DOC-INDEX-FRONTMATTER`)
4. **Example fork passes** (`examples/fork-customization/`) is validated and demonstrates fork extensibility in CI (`AC-TPL-EXAMPLE-FORK-BUILDS`)

**Status:** v3.3.9-kernel is the current IDP-ready closure. All governance surface items wired into specs and CI.

### 8.2 Adoption Definition of Done (Other Repos)

The template is "production ready" when:

1. **At least one service has been built from it** and reached production
2. **Friction log from that service is addressed** (or documented as out-of-scope)
3. **Platform teams can integrate it** via documented APIs and artifacts
4. **New teams can onboard in < 1 hour** with written docs alone

> This is evaluated in forks, not in this repository. Until adoption criteria are met, v3.3.9-kernel remains a stable baseline suitable for early adopters who accept some friction.

---

## 9. Summary

**Current State:**

| Layer | Version | Status |
| ----- | ------- | ------ |
| **Template** | v3.3.14 | Released (tag: v3.3.14) |
| **On main** | v3.3.15 | In progress (30+ commits since v3.3.14) |
| **Kernel** | v3.3.9-kernel | Frozen baseline |
| **Next Minor** | v3.4.0 | External validation release (see §4.5) |
| **Future** | v3.5.0+ | Surface minimization / crate extraction (see §4.6) |

**v3.3.9-kernel** is a stable, selftest-green kernel. All **kernel ACs** (`must_have_ac: true`) pass; non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking selftest.

**v3.3.14** shipped faster precommit defaults, staged-only semantics, targeted spellcheck, unified `/platform/issues` endpoint (PR #74), `issues-search` CLI command, and BDD pagination error contract coverage (PR #76).

**v3.3.15 (on main)** includes:
- Security fixes: DOM-based XSS (PR #146), DoS prevention (PR #116), auth enforcement on GET/HEAD (PR #164)
- 12-step selftest expansion with Docs-as-Code check
- Schema registry compatibility check (`cargo xtask schema-check`)
- Automated release notes via git-cliff integration
- Performance: task board optimization, spawn_blocking for UI operations
- Microcrate architecture: gov-http-types crate, enhanced gov-model with proptest

**v3.4.0** is the "external proof" release: reference IDP consumer, contract tests, and real fork receipts.

**v3.5.0+** is the vision for crate extraction—reducing fork burden by publishing reusable machinery.

**Publishing & Forensics** track (§4.7) runs in parallel, building the truth surface for PR
archaeology and factory improvements. P0 (audit pack docs) is complete.

**Immediate Next Steps:**

1. ✅ v3.3.14 tagged (DevEx improvements + unified issues endpoint)
2. 🔜 Tag v3.3.15 (security fixes + tooling improvements)
3. Collect fork dry-run + AI first-hour receipts (v3.4.0 gate)
4. Fork for real service development
5. Capture friction -> batch improvements into v3.4.0

**What's Still Missing (Honest Assessment):**

- **Green ≠ proven.** Selftest passes, but the template hasn't yet survived real adoption load.
- **API docs are an index, not a guide.** Curl-first examples are a v3.4.0 item.
- **Fork surface is large.** Crate extraction (v3.5.0+) will address this, but only after real friction justifies it.

The recommended path: fork immediately, capture friction, fix what matters, document what you
learned. Don't try to anticipate every need—let real usage tell you what's missing.
