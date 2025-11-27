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
last_updated: 2025-11-26
---

# Roadmap: Rust-as-Spec Platform Cell (v3.3.3)

This document describes the current state of the **v3.3.3 kernel** and what remains to be done.

---

## Philosophy: Opinionated Defaults

This template is **not** a generic Rust starter. It encodes specific opinions about governed, agent-friendly development—enforced through Acceptance Criteria in `specs/spec_ledger.yaml`.

### Core Opinions

The template takes strong positions on:

1. **Environment**: Nix-first (Tier-1) with native OS as Tier-2. Reproducibility over convenience.
2. **CI Gate**: `cargo xtask selftest` must pass. Governance validation is non-negotiable.
3. **Governance Artifacts**: Questions, friction, and forks are first-class, queryable data exposed via APIs.
4. **Agent Surfaces**: `/platform/*` endpoints, context bundles, and `--json` CLI outputs for LLM workflows.

### Why Opinions Matter

Opinions encoded as ACs mean:

- **Fork customization is explicit**: Change the AC, not random config files
- **Upstream feedback is structured**: Report what opinions didn't fit your domain via `docs/how-to/report-fork-feedback.md`
- **Template evolution is evidence-driven**: Friction from real forks drives AC changes

**To customize:** Identify the AC, modify it, update implementation, verify with selftest. See `docs/QUICKSTART.md` "Defaults & Opinions" section for details.

---

## 1. Kernel v3.3.3 Closure

**Status:** Frozen as of 2025-11-27.

The v3.3.3 kernel is **closed**. No new features land here. All work now targets:

- **Forks**: Build services on v3.3.3 baseline; capture friction
- **v3.4.0**: Kernel improvements driven by real fork feedback

This closure is enforced:

- Git tag: `v3.3.3-kernel` marks the frozen baseline
- `ROADMAP.md` updated to reflect closure
- `docs/reference/environment.md` added for clear setup guidance

To fork from v3.3.3, start with `docs/how-to/FIRST_FORK.md`.

---

## 2. Current State (v3.3.3)

The kernel has reached a stable, forkable baseline. All acceptance criteria pass, all selftest gates pass.

### 2.1 Governance Status

| Metric | Value |
|--------|-------|
| **Total ACs** | 65 |
| **Passing** | 65 (100%) |
| **Selftest Gates** | 8/8 passing |
| **Policy Tests** | 22/22 passing |
| **BDD Scenarios** | 110+ passing |

### 2.2 What's Working

**Runtime & APIs:**

- Service health, version, metrics endpoints
- Platform introspection: `/platform/graph`, `/platform/devex/flows`, `/platform/docs/index`, `/platform/schema`
- Agent hints API: `/platform/agent/hints` with task prioritization
- Platform UI dashboard with graph visualization and flows view
- Configuration validation and IAC alignment (Docker Compose, Kubernetes, Terraform)
- Task lifecycle and governance write operations

**DevEx CLI:**

- Development flows: `doctor`, `help-flows`, `check`, `test-changed`, `ac-status`, `ac-coverage`
- Bundler & agent tools: `bundle`, `suggest-next`
- Governance: `adr-new`, `ac-new`, `docs-check`, `graph-export`, `selftest`
- Release management: `release-prepare`, `release-bundle`
- Operational: `audit`, `sbom-local`, `ci-local`, `status`
- Quick validation: `kernel-smoke` (docs-check + selftest)

**Governance:**

- BDD acceptance tests covering all 65 ACs
- Graph invariants for REQ/AC/test/doc relationships
- Policy tests (OPA/Rego) for configuration compliance
- Pre-commit hooks with auto-staging
- AC/ADR bidirectional mapping

### 2.3 Verification

```bash
nix develop
cargo xtask doctor         # Environment validated
cargo xtask selftest       # 8/8 gates pass
cargo xtask ac-status      # 65/65 PASS, 0 FAIL
cargo run -p app-http      # Listening on :8080
```

---

## 3. What's Been Completed (v3.3.3 Polish)

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

---

## 4. What's Still Needed

Only a few items remain - all now have documentation or are external dependencies:

### 4.1 Manual Configuration Required

| Item | Impact | Documentation | Action Required |
|------|--------|---------------|-----------------|
| **Branch protection** | CI can be bypassed | `docs/how-to/setup-branch-protection.md` | Run `.github/scripts/setup-branch-protection.sh` |
| **Tag signing** | Releases not cryptographically verified | `docs/how-to/setup-tag-signing.md` | Set up GPG key and configure Git |

### 4.2 Validation Gap

| Gap | Impact | Effort | Notes |
|-----|--------|--------|-------|
| **No second service validation** | Template assumptions untested in real use | High | Requires building a real service from the template |

### 4.3 External Dependencies

| Item | Impact | Notes |
|------|--------|-------|
| **`lazy-trees` Nix warning** | Cosmetic noise in output | Deprecated Nix 2.30+ setting in Determinate installer config. Documented in TROUBLESHOOTING.md with fix instructions. Safe to ignore. |

---

## 5. Path Forward Options

### 5.1 Option A: Minimal (Lock and Fork)

**Goal:** Freeze the kernel as-is, use it for services, let friction drive improvements.

**Actions:**

1. Configure GitHub branch protection (require selftest, no direct pushes)
   - **Run:** `.github/scripts/setup-branch-protection.sh`
   - **Docs:** `docs/how-to/setup-branch-protection.md`
2. Tag `v3.3.3` as the stable baseline
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

### 5.2 Option B: Consolidate (Fill Documentation Gaps)

**Goal:** Complete documentation before first fork.

**Actions:**

1. Write `docs/explanation/idp-positioning.md` ✅
2. Write `docs/how-to/brownfield-adoption.md` ✅
3. Test and document Windows Tier-2 flow
4. Clean up ADR numbering duplicates
5. Configure branch protection (see `docs/how-to/setup-branch-protection.md`) ✅
6. Then fork

**Timeline:** 1-2 sessions

**Pros:**

- Better onboarding for new teams
- Cleaner starting point
- Reduces "figure it out" friction

**Cons:**

- Delays first real usage
- Documentation may not match reality
- Speculative improvements

### 5.3 Option C: Validate (Second Service First)

**Goal:** Prove the template works before declaring it ready.

**Actions:**

1. Fork for Knowledge Hub immediately
2. Document friction as it occurs
3. Fix kernel issues discovered during use
4. Extract patterns into docs after validation
5. Re-baseline kernel at v3.4.0

**Timeline:** 2-4 sessions

**Pros:**

- Documentation grounded in reality
- Discovers actual problems
- Template improvements earned, not assumed

**Cons:**

- Friction during development
- May need kernel changes mid-service
- More churn before stability

---

## 6. Recommended Path

**Option C (Validate) is recommended** for these reasons:

1. **100% AC pass doesn't mean ready.** The ACs test what we said we'd build, not what services actually need.

2. **Documentation written after use is better.** We'll know what to document because we'll have hit the gaps.

3. **Friction is valuable signal.** The first fork will generate a friction log that tells us exactly what to fix.

### Recommended Sequence

```text
Week 1: Fork for Knowledge Hub
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
        - Batch kernel fixes into v3.4.0

Week 4: Documentation
        - Write docs based on actual experience
        - Update ROADMAP with lessons
        - Release v3.4.0 with fixes + docs
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
cargo xtask fork-register --name "Name" --domain "domain" --kernel-version "v3.3.3" ...
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

The kernel is "fully implemented" when:

1. **At least one service has been built from it** and reached production
2. **Friction log from that service is addressed** (or documented as out-of-scope)
3. **Platform teams can integrate it** via documented APIs and artifacts
4. **New teams can onboard in < 1 hour** with written docs alone

Until then, it's a stable baseline (v3.3.3) suitable for early adopters who accept some friction.

---

## 9. Summary

**v3.3.3** is a stable, selftest-green kernel. All 65 ACs pass. But "selftest green" and "ready for production" are different bars. The gaps are documented above.

The recommended path: fork immediately, capture friction, fix what matters, document what you learned. Don't try to anticipate every need—let real usage tell you what's missing.
