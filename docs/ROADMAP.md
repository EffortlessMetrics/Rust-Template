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
last_updated: 2025-12-27
---

# Roadmap: Rust-as-Spec Platform Cell (v3.3.13)

This document describes the current state of the template (v3.3.13) and the frozen baseline kernel (v3.3.9-kernel tag).

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
|---------|--------|-------|
| v3.3.12 | Tagged | Security hardening, governance architecture, CI improvements |
| v3.3.13 | Tagged | Docs polish + release tooling hardening (see §4.4) |

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
- Selftest gates expanded to 11 steps

To fork from v3.3.9, start with `docs/how-to/FIRST_FORK.md`.

---

## 2. Current State (v3.3.13)

The template is at v3.3.13, building on the frozen v3.3.9-kernel baseline.

- **Kernel ACs** (`must_have_ac: true`): All passing
- **Selftest**: Green (11/11 gates)
- **Non-kernel ACs**: Soft gates, may be UNKNOWN depending on test capture

**Template Version (v3.3.13):**

This is the current active template version with adoption receipts and documentation polish.

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
| **Selftest Gates** | 11/11 passing |
| **Policy Tests** | All passing — run `cargo xtask selftest` for current count |
| **BDD Scenarios** | See `specs/features/` — count tracked in `feature_status.md` |

> **Note:** Selftest only gates on **kernel ACs** (`must_have_ac: true`). Non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking merges. See `docs/feature_status_notes.md` for AC classification details.
>
> **Why no hardcoded counts?** AC/test counts are volatile (depend on which tests ran locally). Only `feature_status.md` (regenerated by `cargo xtask ac-status`) is authoritative. See issue #35.

### 2.2 What's Working

**Runtime & APIs:**

- Service health, version, metrics endpoints
- Platform introspection: `/platform/graph`, `/platform/devex/flows`, `/platform/docs/index`, `/platform/schema`
- Agent hints API: `/platform/agent/hints` with task prioritization
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
cargo xtask selftest       # 11/11 gates pass
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

**Status:** Ready for tag

#### In This Release Candidate

| Item                         | Description                                   | Status              |
| ---------------------------- | --------------------------------------------- | ------------------- |
| **Docs version alignment** | All docs updated to v3.3.13 references | ✅ Merged (PR #40) |
| **Security configuration doc** | Auth modes, CORS, JWT, headers, fail-closed | ✅ Merged (PR #38) |
| **Selftest green** | 11/11 gates passing | ✅ Verified |

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

### 4.5 v3.4.0 – IDP-Ready (Minor Release)

> **Note:** v3.4.0 is the *next minor* kernel closure. The current frozen baseline is `v3.3.9-kernel`.

#### What "IDP-Ready" Means (Contract)

v3.4.0 is not a feature list. It's the point where Backstage/Port consumers can **reliably ingest** `/platform/*` from cells.

| Criterion                             | Definition of Done                                                              |
| ------------------------------------- | ------------------------------------------------------------------------------- |
| **Stable `idp-snapshot` schema**      | Schema versioned; breaking changes require major version bump                   |
| **Example IDP tile(s)**               | At least: governance health tile + docs health tile consuming `/platform/*`    |
| **Multi-service story exists**        | Registry *spec* for multiple cells (even if static YAML initially)             |
| **Receipts-driven hardening loop**    | Friction taxonomy + workflow for promoting soft gates → hard gates             |

#### Entry Criteria (Gate Before Starting v3.4.0)

- v3.3.12 released ✅
- v3.3.13 released with receipts 🔄 (in progress — see §4.4)
- At least one real fork exists and is actively used
- Friction log reviewed; v3.4.0 candidates tagged

#### Planned Work (Demand-Driven)

| Item                                 | Description                                                           | Status     | Priority |
| ------------------------------------ | --------------------------------------------------------------------- | ---------- | -------- |
| **Multi-service registry spec**      | Static YAML registry listing cells and their `idp-snapshot` endpoints | 🔜 Planned | High     |
| **IDP tile reference implementation**| Example Backstage tiles for governance + docs health                  | 🔜 Planned | High     |
| **Friction taxonomy + promotion**    | Workflow for soft → hard gate promotion based on fork feedback        | 🔜 Planned | Medium   |
| **AI agent feedback loop**           | Structured agent → friction → kernel improvement cycle                | 🔜 Planned | Medium   |

#### Deferred to v3.5.0+

These belong in later releases unless adoption pressure forces them earlier:

| Item                                 | Description                                                           | Rationale                        |
| ------------------------------------ | --------------------------------------------------------------------- | -------------------------------- |
| **Cross-cell graph queries**         | Query governance state across multiple cells                          | Needs registry + real multi-cell usage |
| **Advanced policy packs**            | PCI-DSS, HIPAA compliance templates                                   | Domain-specific; not core        |
| **Fleet-wide Backstage integration** | Plugin reading `/platform/*` from N services                          | v3.5.0 after registry is proven  |

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

See [v3.4.0-plan.md](archive/v3.4.0-plan.md) for scope when v3.4.0 work begins.

---

## 5. Path Forward Options

> **Current State (v3.3.13):** The template has completed security hardening, architecture refactoring, and documentation polish. The next step is external validation via fork receipts. See §4.4 for v3.3.13 release checklist.

### 5.1 Option A: Minimal (Lock and Fork) — *Active*

**Goal:** Freeze the kernel as-is, use it for services, let friction drive improvements.

**Status:** This is the current path. v3.3.9-kernel is frozen; v3.3.13 adds validation receipts.

**Immediate Next Steps:**

1. ✅ Kernel frozen at v3.3.9-kernel
2. ✅ Branch protection configured
3. ✅ Documentation complete (v3.3.12 → v3.3.13)
4. 🔜 Create fork from v3.3.9-kernel tag
5. 🔜 Complete fork dry-run and AI first-hour receipts
6. 🔜 Tag v3.3.13 with evidence bundle

**After v3.3.13:**

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

* Documentation grounded in reality
* Discovers actual problems
* Template improvements earned, not assumed

**Cons:**

* Friction during development
* May need kernel changes mid-service
* More churn before stability

---

## 5A. Adoption Phases (Build-Out Track)

> **Scope:** These phases happen **in forks and platform infrastructure**, not in this kernel repo. Kernel improvements are batched at phase gates based on demand, not speculation.

### Phase 1: Fork Usage Baseline

**Goal:** Test kernel invariants in real service environment

**Activities:**

* Fork template into new service using `v3.3.9-kernel` tag
* Run `nix develop && cargo xtask doctor && cargo xtask selftest` to validate baseline
* Wire in service identity via `specs/service_metadata.yaml`
* Add domain stories/REQs/ACs to `specs/spec_ledger.yaml`
* Capture friction in fork's `FRICTION_LOG.md` and surface via `/platform/friction` API
* Use `docs/how-to/report-fork-feedback.md` to report kernel issues upstream

**Success Criteria:**

* Selftest green on every PR in fork repo
* Friction systematically logged and categorized
* Service builds and runs with template governance intact

### Phase 2: IDP Tile Integration

**Goal:** Surface governance and docs health in developer portal

**Activities:**

* Build **Governance Health tile** using `/platform/status` endpoint

  * Show AC pass/fail counts, policy status, selftest gate results
* Build **Docs Health tile** using `/platform/docs/index` endpoint

  * Show doc types, coverage, staleness, missing entries
* (Optional) Add **Task/Hints tile** using `/platform/agent/hints` endpoint

  * Surface prioritized work items for teams and agents
* Reference `docs/explanation/json-contracts.md` for JSON schema contracts
* Validate tile data against fork services (Phase 1 outputs)

**Success Criteria:**

* Template-based services visible in IDP with health metrics
* Teams can see governance drift in real-time
* Documentation health surfaced without manual audits

### Phase 3: Governed Agent Pilot

**Goal:** Validate kernel is truly agent-friendly

**Activities:**

* Deploy Claude Code agents to 2-3 fork repos from Phase 1
* Use Skills: `bootstrap-dev-env`, `governed-feature-dev`, `governed-maintenance`
* Agent workflow loop:

  1. Query `/platform/agent/hints` for prioritized tasks
  2. Generate context bundle via `cargo xtask bundle <task_name>`
  3. Edit code/tests/docs within bundle scope
  4. Validate with `cargo xtask test-ac <AC_ID>`
  5. Gate on `cargo xtask selftest` before PR
* Require AC/REQ/Doc invariants green (`docs-check` + `selftest`)
* Capture agent friction separately from human developer friction

**Success Criteria:**

* Agents productive in 2-3 real service repos without human intervention
* Agent-generated PRs pass selftest on first attempt >80% of the time
* Clear friction log distinguishing agent vs. human developer pain points

### Phase 4: Kernel vNext (Demand-Driven)

**Goal:** Batch real feedback into next kernel version

**Activities:**

* Review friction logs from Phases 1-3 (fork usage + IDP + agents)
* Categorize feedback:

  * **Kernel fixes:** gaps in `spec_ledger.yaml`, broken contracts, missing flows
  * **Soft → hard promotions:** checks validated in real usage, ready to gate
  * **JSON contract refinements:** IDP/agent usage reveals schema gaps
  * **Out-of-scope:** fork-specific needs, not generalizable
* Promote soft checks to hard gates after validation (e.g., `docs-check` strictness)
* Refine JSON contracts based on IDP tile and agent integration patterns
* Implement versioning engine refactor if `release-prepare` friction is systematic
* Add new patterns discovered in forks (e.g., common service types, IAC extensions)

**Success Criteria:**

* v3.4.0 (or v4.0.0) released with changes **driven by friction**, not speculation
* All promoted hard gates have evidence from ≥2 fork repos
* JSON contracts validated by real IDP/agent consumers
* Kernel changelog clearly attributes improvements to fork feedback

---

**Note:** These phases are **adoption-driven** and happen outside this kernel repository. Kernel improvements are batched at phase gates (especially Phase 4) rather than landed speculatively. The v3.3.9-kernel is complete; validation and evolution happen through real usage.

---

## 6. Recommended Path (For Adopters)

> **Scope note:** This section describes what happens **in forks**, not in this repo.
> The kernel (v3.3.9) is frozen. Template is at v3.3.13. Validation happens when you use it.

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
# Platform APIs for agents
curl localhost:8080/platform/agent/hints   # Prioritized task hints
curl localhost:8080/platform/questions     # Question artifacts
curl localhost:8080/platform/forks         # Fork registry
curl localhost:8080/platform/friction      # Friction log
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

   * Version alignment (`AC-PLT-009`, `AC-PLT-010`, extended with feature-status invariants)
   * `doc_index.yaml` ↔ front-matter sync (`AC-PLT-DOC-INDEX-FRONTMATTER`)
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

| Layer        | Version         | Status                          |
|--------------|-----------------|--------------------------------|
| **Template** | v3.3.13         | Release candidate in progress  |
| **Kernel**   | v3.3.9-kernel   | Frozen baseline                |

**v3.3.9-kernel** is a stable, selftest-green kernel. All **kernel ACs** (`must_have_ac: true`) pass; non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking selftest.

**v3.3.13** adds documentation polish, version alignment, and security configuration docs. The remaining gates are external validation receipts (fork dry-run + AI first-hour).

**Next Steps:**

1. Complete fork validation receipts (see §4.4 Release Checklist)
2. Tag v3.3.13 with evidence bundle
3. Fork for real service development
4. Capture friction → batch improvements into v3.4.0

The recommended path: fork immediately, capture friction, fix what matters, document what you learned. Don't try to anticipate every need—let real usage tell you what's missing.
