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
last_updated: 2025-12-01
---

# Roadmap: Rust-as-Spec Platform Cell (v3.3.7)

This document describes the current state of the **v3.3.7 kernel** and what remains to be done.

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

## 1. Kernel v3.3.4 Closure (Historical)

**Status:** Frozen as of 2025-11-30.

v3.3.3 was the first kernel closure. v3.3.4 is a patch-level closure that stabilizes:

- **BDD harness exit semantics** (`AC-TPL-BDD-EXIT-CODES`)
- **Agent hint schema** (`AC-TPL-AGENT-HINTS-SCHEMA`)
- **Bundle manifest linkage** (`AC-TPL-BUNDLE-MANIFEST-LINKED`, `AC-TPL-BUNDLE-MINIMAL-SCOPE`)
- **SPEC_ROOT contract** for xtask commands

No new features land here. All forward work targets:

- **Forks**: Build services on v3.3.4 baseline; capture friction
- **v3.4.0**: Kernel improvements driven by real fork feedback

This closure is enforced:

- Git tag: `v3.3.4-kernel` marks the frozen baseline
- `ROADMAP.md` updated to reflect closure
- Selftest gates expanded to 11 steps

To fork from v3.3.4, start with `docs/how-to/FIRST_FORK.md`.

---

## 2. Current State (v3.3.7)

The kernel has reached a stable, forkable baseline. All acceptance criteria pass, all selftest gates pass.

**v3.3.7 Highlights:**

- **Manifest-driven versioning engine** – `specs/version_manifest.yaml` declares all version-bearing files; `release-prepare` uses this manifest to update versions atomically
- **Versioning ACs enforced** – `AC-TPL-VERSION-MANIFEST`, `AC-TPL-VERSION-DRYRUN`, `AC-TPL-VERSION-ATOMIC` promoted from deferred to kernel contracts
- **`--dry-run` support** – Preview all version changes before applying them

### 2.1 Governance Status

| Metric | Value |
|--------|-------|
| **Kernel ACs** | All passing (see `docs/feature_status_notes.md` for current count) |
| **Non-kernel ACs** | Soft gates (tracked, not enforced) |
| **Selftest Gates** | 11/11 passing |
| **Policy Tests** | 22/22 passing |
| **BDD Scenarios** | 170+ passing |

> **Note:** Selftest only gates on **kernel ACs** (`must_have_ac: true`). Non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking merges. See `docs/feature_status_notes.md` for exact counts and AC classification details.

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
- **Docs-as-Code v2:** `spec_ledger.yaml` is the version authority; `docs-check` enforces alignment across 8 consumer files; `xtask version --json` is the machine surface

**v3.3.4 Additions:**

- **BDD Harness**: Semantic exit detection via `is_bdd_success()` - stable across runs
- **Hint Schema**: `/platform/agent/hints` and `xtask suggest-next --format json` share canonical Hint types
- **Bundle Manifests**: `bundle.yaml` links to REQs/ACs/tests with soft scope audit

### 2.3 Verification

```bash
nix develop
cargo xtask doctor         # Environment validated
cargo xtask selftest       # 11/11 gates pass
cargo xtask ac-status      # All kernel ACs PASS
cargo run -p app-http      # Listening on :8080
```

> **Version authority:** `specs/spec_ledger.yaml → metadata.template_version` is the canonical kernel version. All other docs (README, CLAUDE, ROADMAP, KERNEL_SNAPSHOT, TEMPLATE-CONTRACTS, service_metadata, doc_index, CHANGELOG) are validated against it by `cargo xtask docs-check`.

---

## 3. What's Been Completed (3.3.x Polish)

The following gaps have been addressed:

### 3.1 Documentation ✅

| Item | Status | Location |
|------|--------|----------|
| **IDP positioning doc** | ✅ Complete | `docs/explanation/idp-positioning.md` |
| **Brownfield adoption guide** | ✅ Complete | `docs/guides/brownfield-adoption.md` |
| **Fork feedback workflow** | ✅ Complete | `docs/how-to/report-fork-feedback.md` |
| **Quick start guide** | ✅ Complete | `docs/QUICKSTART.md` |
| **Troubleshooting guide** | ✅ Complete | `docs/TROUBLESHOOTING.md` |
| **Windows development guide** | ✅ Complete | `docs/how-to/windows-development.md` |
| **CI workflows reference** | ✅ Complete | `docs/reference/ci-workflows.md` |
| **Branch protection setup** | ✅ Complete | `docs/how-to/setup-branch-protection.md` |
| **Tag signing setup** | ✅ Complete | `docs/how-to/setup-tag-signing.md` |
| **Environment setup guide** | ✅ Complete | `docs/reference/environment.md` |

### 3.2 Operational Tooling ✅

| Item | Status | Command/Endpoint |
|------|--------|------------------|
| **Questions-as-artifacts** | ✅ Complete | `cargo xtask question-new`, `questions-list` |
| **Friction log API** | ✅ Complete | `GET /platform/friction`, `/platform/friction/{id}` |
| **Fork registry** | ✅ Complete | `cargo xtask fork-list`, `fork-register` |
| **Version command** | ✅ Complete | `cargo xtask version` (with `--json`) |
| **Friction CLI** | ✅ Complete | `cargo xtask friction-new`, `friction-list` |
| **Release AC deltas** | ✅ Complete | `cargo xtask release-bundle` now includes AC changes |
| **Branch protection script** | ✅ Complete | `.github/scripts/setup-branch-protection.sh` |
| **Service-init command** | ✅ Complete | `cargo xtask service-init` - single command fork branding |
| **JSON CLI outputs** | ✅ Complete | `--json` flag on `ac-status`, `friction-list`, `questions-list`, `fork-list` |
| **Questions HTTP API** | ✅ Complete | `GET /platform/questions`, `/platform/questions/{id}` |
| **Forks HTTP API** | ✅ Complete | `GET /platform/forks`, `/platform/forks/{name}` |

### 3.3 Editor Integration ✅

| Item | Status | Location |
|------|--------|----------|
| **VS Code extensions** | ✅ Complete | `.vscode/extensions.json` |
| **VS Code tasks** | ✅ Complete | `.vscode/tasks.json` |
| **VS Code launch configs** | ✅ Complete | `.vscode/launch.json` |
| **VS Code settings** | ✅ Complete | `.vscode/settings.json` |

### 3.4 Test Isolation ✅

| Item | Status | Notes |
|------|--------|-------|
| **BDD test isolation** | ✅ Fixed | Tests no longer pollute tracked files |

### 3.5 Technical Debt ✅

| Item | Status | Notes |
|------|--------|-------|
| **ADR numbering duplicates** | ✅ Fixed | Removed test scaffolds, renumbered 0007→0019 |
| **Release evidence incomplete** | ✅ Fixed | AC delta tracking added to release-bundle |

### 3.6 v3.3.4 Polish (Harness, Hints, Bundles) ✅

| Item | Status | Notes |
|------|--------|-------|
| **BDD harness semantics** | ✅ Complete | `bdd.rs` + `is_bdd_success()` (`AC-TPL-BDD-EXIT-CODES`) |
| **Agent Hint schema** | ✅ Complete | `AC-TPL-AGENT-HINTS-SCHEMA` + BDD + unit tests |
| **Bundle manifest v1.5** | ✅ Complete | `AC-TPL-BUNDLE-MANIFEST-LINKED` + `AC-TPL-BUNDLE-MINIMAL-SCOPE` |
| **SPEC_ROOT contract** | ✅ Complete | `AC-TPL-XTASK-SPEC-ROOT` — `spec_root()` honors env var |
| **Pre-commit auto-fix** | ✅ Complete | Hook runs `xtask precommit`, auto-fixes fmt/skills/feature_status |
| **Docs-as-Code v2** | ✅ Complete | `AC-PLT-009` + `AC-PLT-010` — see `docs/explanation/TEMPLATE-CONTRACTS.md` |

---

## 4. What's Still Needed

Only a few items remain - all now have documentation or are external dependencies:

### 4.1 Manual Configuration Required

| Item | Impact | Documentation | Action Required |
|------|--------|---------------|-----------------|
| **Branch protection** | CI can be bypassed | `docs/how-to/setup-branch-protection.md` | Run `.github/scripts/setup-branch-protection.sh` |
| **Tag signing** | Releases not cryptographically verified | `docs/how-to/setup-tag-signing.md` | Set up GPG key and configure Git |

### 4.2 External Validation (Out of Scope for This Repo)

| Gap | Impact | Effort | Notes |
|-----|--------|--------|-------|
| **No second service built yet** | Template assumptions untested in a real fork | High | Requires building a service in another repo using this template as a baseline |

> This is intentionally **outside the scope of this repository**. This roadmap tracks it as a recommended next step for adopters, not as work to be done here. The kernel is complete; validation happens in forks.

### 4.3 External Dependencies

| Item | Impact | Notes |
|------|--------|-------|
| **`lazy-trees` Nix warning** | Cosmetic noise in output | Deprecated Nix 2.30+ setting in Determinate installer config. Documented in TROUBLESHOOTING.md with fix instructions. Safe to ignore. |

### 4.4 v3.4.0 – Planned (Future Kernel Closure)

> **Note:** v3.4.0 is the *next planned* kernel closure. The current frozen baseline is `v3.3.7-kernel`.
> Items below represent future work driven by fork friction, not work completed in this kernel.

| Item | Description | Status |
|------|-------------|--------|
| **Multi-service orchestration** | Cross-service governance coordination | 🔜 Planned |
| **Advanced policy packs** | PCI-DSS, HIPAA compliance templates | 🔜 Planned |
| **Fleet-wide Backstage integration** | Plugin reading `/platform/*` from multiple services | 🔜 Planned |

Items completed in v3.3.7 (now part of the frozen kernel):

| Item | Description | Status |
|------|-------------|--------|
| **IDP Snapshot Contract** | `cargo xtask idp-snapshot` + `/platform/idp/snapshot` emit stable JSON | ✅ In v3.3.7 |
| **Platform Schema** | `specs/platform_schema.yaml` defines all endpoint responses | ✅ In v3.3.7 |
| **Docs-as-Code v3** | Bidirectional `doc_index.yaml` ↔ front-matter sync | ✅ In v3.3.7 |
| **Feature-status invariants** | `feature_status.md` header matches spec_ledger version | ✅ In v3.3.7 |
| **Centralized Env Helpers** | `crate::env::is_ci()`, `is_noninteractive()` reduce duplication | ✅ In v3.3.7 |
| **Example fork CI** | `examples/fork-customization/` with CI template | ✅ In v3.3.7 |

See [v3.4.0-plan.md](v3.4.0-plan.md) for scope when v3.4.0 work begins.

---

## 5. Path Forward Options

### 5.1 Option A: Minimal (Lock and Fork)

**Goal:** Freeze the kernel as-is, use it for services, let friction drive improvements.

**Actions:**

1. Configure GitHub branch protection (require selftest, no direct pushes)
   - **Run:** `.github/scripts/setup-branch-protection.sh`
   - **Docs:** `docs/how-to/setup-branch-protection.md`
2. Tag the current kernel:

   ```bash
   git tag "$(cargo xtask version --json | jq -r .kernel_tag)" -m "Kernel closure"
   ```

3. Fork for Knowledge Hub or other service
4. Capture friction in `FRICTION_LOG.md`
5. Only update kernel when friction is systematic

**Timeline:** Immediate

**Pros:**

- Fastest path to value
- Real usage reveals actual gaps
- Avoids over-engineering

**Cons:**

- Known gaps remain
- Documentation incomplete
- May hit issues in first fork

### 5.2 Option B: Consolidate (Fill Documentation Gaps) — *Historical*

> **Note:** Option B was largely executed during the 3.3.3 → 3.3.4 cycle. It's kept here to explain how we closed the documentation gaps.

**Goal:** Complete documentation before first fork.

**Actions (mostly done in v3.3.4):**

1. Write `docs/explanation/idp-positioning.md` ✅
2. Write `docs/how-to/brownfield-adoption.md` ✅
3. Configure branch protection (see `docs/how-to/setup-branch-protection.md`) ✅
4. Windows Tier-2 flow documented ✅ (see `docs/how-to/windows-development.md`)
5. ADR numbering duplicates cleaned up ✅
6. (Optional ongoing) Keep Tier-2 Windows notes fresh as friction is discovered

**Timeline:** Mostly complete; ongoing maintenance only

**Pros:**

- Better onboarding for new teams
- Cleaner starting point
- Reduces "figure it out" friction

**Cons:**

- Delays first real usage
- Documentation may not match reality
- Speculative improvements

### 5.3 Option C: Validate (Second Service First) — **Adoption Track, Not Kernel Work**

> This option describes how to use the template in a **separate service repository**.
> It does not require changes to this repository once `v3.3.4-kernel` is tagged.

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

- Fork template into new service using `v3.3.7-kernel` tag
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

**Note:** These phases are **adoption-driven** and happen outside this kernel repository. Kernel improvements are batched at phase gates (especially Phase 4) rather than landed speculatively. The v3.3.7-kernel is complete; validation and evolution happen through real usage.

---

## 6. Recommended Path (For Adopters)

> **Scope note:** This section describes what happens **in forks**, not in this repo.
> The kernel (v3.3.5) is complete. Validation happens when you use it.

**Option C (Validate) is recommended** for these reasons:

1. **100% AC pass doesn't mean ready.** The ACs test what we said we'd build, not what services actually need.

2. **Documentation written after use is better.** We'll know what to document because we'll have hit the gaps.

3. **Friction is valuable signal.** The first fork will generate a friction log that tells us exactly what to fix.

### Recommended Sequence (In Your Fork)

```text
Week 1: Fork for your service
        - Use only documented flows and commands
        - Capture friction immediately
        - Don't fix kernel, just document

Week 2: Service development
        - Add domain REQs/ACs to fork
        - Use bundler and agent hints
        - Continue friction capture

Week 3: Kernel retrospective
        - Review friction log
        - Categorize: kernel fix vs. service-specific vs. doc-only
        - Report kernel issues via docs/how-to/report-fork-feedback.md

Week 4: Documentation
        - Write docs based on actual experience
        - Kernel fixes (if any) batch into v3.4.0 here
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
   - Version alignment (`AC-PLT-009`, `AC-PLT-010`, extended with feature-status invariants)
   - `doc_index.yaml` ↔ front-matter sync (`AC-PLT-DOC-INDEX-FRONTMATTER`)
4. **Example fork passes** (`examples/fork-customization/`) is validated and demonstrates fork extensibility in CI (`AC-TPL-EXAMPLE-FORK-BUILDS`)

**Status:** v3.3.7-kernel is the current IDP-ready closure. All governance surface items wired into specs and CI.

### 8.2 Adoption Definition of Done (Other Repos)

The template is "production ready" when:

1. **At least one service has been built from it** and reached production
2. **Friction log from that service is addressed** (or documented as out-of-scope)
3. **Platform teams can integrate it** via documented APIs and artifacts
4. **New teams can onboard in < 1 hour** with written docs alone

> This is evaluated in forks, not in this repository. Until adoption criteria are met, v3.3.7-kernel remains a stable baseline suitable for early adopters who accept some friction.

---

## 9. Summary

**v3.3.7-kernel** is a stable, selftest-green kernel. All **kernel ACs** (`must_have_ac: true`) pass; non-kernel ACs are tracked as soft gates and may be failing or unknown without blocking selftest. But "selftest green" and "ready for production" are different bars. The gaps are documented above.

The recommended path: fork immediately, capture friction, fix what matters, document what you learned. Don't try to anticipate every need—let real usage tell you what's missing.
