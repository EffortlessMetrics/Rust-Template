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

## 1. Current State (v3.3.3)

The kernel has reached a stable, forkable baseline. All acceptance criteria pass, all selftest gates pass.

### 1.1 Governance Status

| Metric | Value |
|--------|-------|
| **Total ACs** | 65 |
| **Passing** | 65 (100%) |
| **Selftest Gates** | 8/8 passing |
| **Policy Tests** | 22/22 passing |
| **BDD Scenarios** | 110+ passing |

### 1.2 What's Working

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

### 1.3 Verification

```bash
nix develop
cargo xtask doctor         # Environment validated
cargo xtask selftest       # 8/8 gates pass
cargo xtask ac-status      # 65/65 PASS, 0 FAIL
cargo run -p app-http      # Listening on :8080
```

---

## 2. What's Been Completed (v3.3.3 Polish)

The following gaps have been addressed:

### 2.1 Documentation ✅

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

### 2.2 Operational Tooling ✅

| Item | Status | Command/Endpoint |
|------|--------|------------------|
| **Questions-as-artifacts** | ✅ Complete | `cargo xtask question-new`, `questions-list` |
| **Friction log API** | ✅ Complete | `GET /platform/friction`, `/platform/friction/{id}` |
| **Fork registry** | ✅ Complete | `cargo xtask fork-list`, `fork-register` |
| **Version command** | ✅ Complete | `cargo xtask version` (with `--json`) |
| **Friction CLI** | ✅ Complete | `cargo xtask friction-new`, `friction-list` |
| **Release AC deltas** | ✅ Complete | `cargo xtask release-bundle` now includes AC changes |
| **Branch protection script** | ✅ Complete | `.github/scripts/setup-branch-protection.sh` |

### 2.3 Test Isolation ✅

| Item | Status | Notes |
|------|--------|-------|
| **BDD test isolation** | ✅ Fixed | Tests no longer pollute tracked files |

### 2.4 Technical Debt ✅

| Item | Status | Notes |
|------|--------|-------|
| **ADR numbering duplicates** | ✅ Fixed | Removed test scaffolds, renumbered 0007→0019 |
| **Release evidence incomplete** | ✅ Fixed | AC delta tracking added to release-bundle |

---

## 3. What's Still Needed

Only a few items remain - all now have documentation or are external dependencies:

### 3.1 Manual Configuration Required

| Item | Impact | Documentation | Action Required |
|------|--------|---------------|-----------------|
| **Branch protection** | CI can be bypassed | `docs/how-to/setup-branch-protection.md` | Run `.github/scripts/setup-branch-protection.sh` |
| **Tag signing** | Releases not cryptographically verified | `docs/how-to/setup-tag-signing.md` | Set up GPG key and configure Git |

### 3.2 Validation Gap

| Gap | Impact | Effort | Notes |
|-----|--------|--------|-------|
| **No second service validation** | Template assumptions untested in real use | High | Requires building a real service from the template |

### 3.3 External Dependencies

| Item | Impact | Notes |
|------|--------|-------|
| **`lazy-trees` Nix warning** | Cosmetic noise in output | Deprecated Nix 2.30+ setting in Determinate installer config. Documented in TROUBLESHOOTING.md with fix instructions. Safe to ignore. |

---

## 4. Path Forward Options

### Option A: Minimal (Lock and Fork)

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

### Option B: Consolidate (Fill Documentation Gaps)

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

### Option C: Validate (Second Service First)

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

## 5. Recommended Path

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

## 6. Quick Reference

### Day-0 Commands (New Clone)

```bash
nix develop                    # Enter devshell
cargo xtask doctor             # Verify environment
cargo xtask kernel-smoke       # Quick validation
cargo run -p app-http          # Start service
```

### Development Commands

```bash
cargo xtask check              # Fast local checks
cargo xtask test-changed       # Run affected tests
cargo xtask test-ac AC-XXX     # Test specific AC
cargo xtask ac-status          # View AC coverage
cargo xtask selftest           # Full governance check
cargo xtask version            # Show kernel version
```

### Governance Artifacts

```bash
cargo xtask friction-new --category X --severity Y --summary "..."
cargo xtask friction-list      # List friction entries
cargo xtask question-new --category X --summary "..." --flow F --phase P --description "..."
cargo xtask questions-list     # List questions
cargo xtask fork-register --name "Name" --domain "domain" --kernel-version "v3.3.3" ...
cargo xtask fork-list          # List registered forks
```

### Agent Workflow

```bash
cargo xtask bundle implement_ac    # Get context bundle
cargo xtask suggest-next           # Get task suggestions
curl localhost:8080/platform/agent/hints  # API-based hints
```

### Release Commands

```bash
cargo xtask release-prepare X.Y.Z
cargo xtask selftest
cargo xtask release-bundle X.Y.Z
```

---

## 7. Definition of Done

The kernel is "fully implemented" when:

1. **At least one service has been built from it** and reached production
2. **Friction log from that service is addressed** (or documented as out-of-scope)
3. **Platform teams can integrate it** via documented APIs and artifacts
4. **New teams can onboard in < 1 hour** with written docs alone

Until then, it's a stable baseline (v3.3.3) suitable for early adopters who accept some friction.

---

## 8. Summary

**v3.3.3** is a stable, selftest-green kernel. All 65 ACs pass. But "selftest green" and "ready for production" are different bars. The gaps are documented above.

The recommended path: fork immediately, capture friction, fix what matters, document what you learned. Don't try to anticipate every need—let real usage tell you what's missing.
