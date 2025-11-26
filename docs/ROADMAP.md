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

## 2. What's Still Needed

Despite 100% AC coverage, there are gaps between "selftest green" and "production-ready platform cell":

### 2.1 Enforcement Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| **Branch protection not configured** | Anyone can push directly to main, bypassing CI | Low (manual GitHub config) |
| **No required status checks** | PRs can merge without selftest passing | Low (manual GitHub config) |
| **Tag signing not enforced** | Release tags can be created without verification | Medium |

### 2.2 Documentation Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| **No IDP positioning doc** | Platform teams don't know how to integrate | Medium |
| **No brownfield migration guide** | Teams with existing services can't adopt | Medium |
| **Windows Tier-2 not fully tested** | Windows users may hit undocumented issues | Low-Medium |

### 2.3 Operational Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| **No second service validation** | Template assumptions untested in real use | High (requires consuming service) |
| **Questions-as-artifacts not implemented** | Agents can't record ambiguity systematically | Medium |
| **Friction log entries not surfaced** | Systemic issues not visible in governance | Low |

### 2.4 Technical Debt

| Item | Impact | Effort |
|------|--------|--------|
| **`lazy-trees` Nix warning** | Cosmetic noise in output | External (Nix config) |
| **ADR numbering duplicates** | ADRs 0007-0016 have duplicates | Low (cleanup) |
| **Release evidence incomplete** | Missing AC deltas between versions | Medium |

---

## 3. Path Forward Options

### Option A: Minimal (Lock and Fork)

**Goal:** Freeze the kernel as-is, use it for services, let friction drive improvements.

**Actions:**

1. Configure GitHub branch protection (require selftest, no direct pushes)
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

1. Write `docs/explanation/idp-positioning.md`
2. Write `docs/how-to/brownfield-adoption.md`
3. Test and document Windows Tier-2 flow
4. Clean up ADR numbering duplicates
5. Configure branch protection
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

## 4. Recommended Path

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

## 5. Quick Reference

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

## 6. Definition of Done

The kernel is "fully implemented" when:

1. **At least one service has been built from it** and reached production
2. **Friction log from that service is addressed** (or documented as out-of-scope)
3. **Platform teams can integrate it** via documented APIs and artifacts
4. **New teams can onboard in < 1 hour** with written docs alone

Until then, it's a stable baseline (v3.3.3) suitable for early adopters who accept some friction.

---

## 7. Summary

**v3.3.3** is a stable, selftest-green kernel. All 65 ACs pass. But "selftest green" and "ready for production" are different bars. The gaps are documented above.

The recommended path: fork immediately, capture friction, fix what matters, document what you learned. Don't try to anticipate every need—let real usage tell you what's missing.
