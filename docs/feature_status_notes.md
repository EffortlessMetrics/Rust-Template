# Feature Status Notes

**Last Updated:** 2025-11-26
**Template Version:** v3.3.1
**Purpose:** Document AC normalization, test hygiene, and distinguish template future work from kernel requirements.

---

## Executive Summary

As of 2025-11-26:
- **Total ACs:** 65
- **Kernel ACs (must_have_ac: true):** 54
- **Future ACs (must_have_ac: false, tags: [future]):** 11
- **Currently Passing:** 35 ACs
- **Currently Failing:** 30 ACs (27 kernel, 3 future)

**Key Finding:** Most ACs have only **1 test per AC** (single-test coverage). This is acceptable for initial implementation but represents a test diversity gap that should be addressed incrementally.

---

## 1. Failing Kernel ACs (27)

These are **kernel requirements** (`must_have_ac: true`, `tags: [kernel]`) that are currently failing. These MUST be implemented to achieve selftest-green status.

### DevEx Platform Commands (REQ-PLT-*)
- **AC-PLT-001**: `cargo xtask doctor` - Environment validation
- **AC-PLT-002**: `cargo xtask help-flows` - Command map rendering
- **AC-PLT-003**: `cargo xtask check` - Fast dev loop (fmt + clippy + tests)
- **AC-PLT-004**: `cargo xtask adr-new` - ADR scaffolding
- **AC-PLT-005**: `cargo xtask ac-new` - AC scaffolding with duplicate detection
- **AC-PLT-006**: `cargo xtask audit` - Security audit (cargo-audit + cargo-deny)
- **AC-PLT-007**: Audit recovery guidance (4-step recovery on failure)
- **AC-PLT-008**: `cargo xtask sbom-local` - SBOM generation
- **AC-PLT-009**: `cargo xtask docs-check` - Version alignment validation
- **AC-PLT-010**: Docs-check regenerates feature_status.md and checks git tree
- **AC-PLT-011**: `cargo xtask release-prepare X.Y.Z` - Release version updates
- **AC-PLT-012**: `cargo xtask release-verify` - Full release validation
- **AC-PLT-013**: Release-verify provides git command sequence on success
- **AC-PLT-015**: Selftest enforces devex contract (required commands exist)
- **AC-PLT-016**: `cargo xtask ci-local` - Orchestrate full CI locally
- **AC-PLT-017**: `cargo xtask status` - Governance status dashboard
- **AC-PLT-019**: Selftest displays condensed summary for all 8 steps

### Platform Features
- **AC-TPL-FLOW-IDEMPOTENT**: Flows are safe to rerun (idempotency)
- **AC-TPL-GRAPH-MERMAID**: Graph export to Mermaid format
- **AC-TPL-GRAPH-SELFTEST**: Selftest validates graph invariants
- **AC-TPL-HOOKS-INSTALL**: Install Git pre-commit hooks

### Release Management
- **AC-TPL-REL-EVIDENCE**: Release bundle generation (release_evidence/vX.Y.Z.md)
- **AC-TPL-REL-CHANGELOG**: Evidence includes structured sections for changelog

### Agent Interface
- **AC-TPL-SKILLS-ALIGN-001**: Skills aligned with documented workflows
- **AC-TPL-SKILLS-FMT**: `cargo xtask skills-fmt` - Normalize SKILL.md files
- **AC-TPL-SKILLS-LINT**: `cargo xtask skills-lint` - Validate skills frontmatter

**Recommendation:** Prioritize implementing these in the order suggested by `/platform/agent/hints` endpoint. Focus on foundation (doctor, check, help-flows) → design scaffolding (adr-new, ac-new) → security (audit) → release (release-prepare, release-verify) → skills tooling.

---

## 2. Failing Future ACs (4)

These are **informational/future work** (`must_have_ac: false`, `tags: [future]`) that are NOT blocking selftest. They represent planned extensions.

- **AC-TPL-SUGGEST-NEXT-CLI**: `cargo xtask suggest-next --task <ID>` - CLI task suggestions
- **AC-TPL-TASKS-CLI**: `cargo xtask tasks-list` - List tasks from tasks.yaml
- **AC-TPL-TASKS-CREATE-CLI**: `cargo xtask task-create` - Create new tasks
- **AC-TPL-TASKS-UPDATE-CLI**: `cargo xtask task-update` - Update task status

**Status:** These are template roadmap items. The HTTP equivalents (`/platform/tasks`, `/platform/agent/hints`) are implemented and passing. CLI versions are deferred.

**Recommendation:** Document as "Template Future Work" in TEMPLATE-CONTRACTS.md. Do not treat as failures—these are intentionally deferred features.

---

## 3. Test Diversity Gap

**Current State:** 59 of 65 ACs have **single-test coverage** (1 test per AC).

**ACs with Multiple Tests (Good Diversity):**
- AC-TPL-CONFIG-VALIDATION: 2 / 2 (BDD + unit)
- AC-TPL-GRAPH-MERMAID: 2 / 2 (BDD + unit)
- AC-TPL-LOG-NO-SECRETS: 2 / 2 (BDD + unit)
- AC-TPL-TASK-TRANSITIONS: 2 / 2 (unit: allowed + forbidden)

**Why This Matters:**
- Single-test ACs are vulnerable to false positives
- Multiple test types (BDD scenarios, unit tests, integration tests) provide confidence
- Particularly important for kernel contracts with complex behavior

**Recommendation:**
1. **Immediate:** Accept single-test coverage for initial implementation
2. **Short-term (next sprint):** Add second test scenarios for high-risk kernel ACs:
   - Security ACs (AC-PLT-006, AC-PLT-007, AC-PLT-008)
   - Release ACs (AC-PLT-011, AC-PLT-012, AC-PLT-013)
   - Graph invariants (AC-TPL-GRAPH-SELFTEST, AC-TPL-GRAPH-MERMAID)
3. **Long-term:** Target 2-3 test scenarios per kernel AC as standard practice

**Test Diversity Roadmap** (see §4 below).

---

## 4. Test Diversity Roadmap

This section outlines the plan to incrementally improve test coverage and diversity for kernel ACs.

### Phase 1: Critical Path (Priority 1)
Target: Security, Release, Graph Invariants

| AC ID | Current Coverage | Target Coverage | Rationale |
|-------|------------------|-----------------|-----------|
| AC-PLT-006 | 1 BDD | 1 BDD + 1 unit (deny.toml parsing) | Security-critical |
| AC-PLT-007 | 1 BDD | 1 BDD + 1 unit (recovery message format) | Security guidance |
| AC-PLT-008 | 1 BDD | 1 BDD + 1 unit (SBOM validation) | Supply chain |
| AC-PLT-011 | 1 BDD | 1 BDD + 1 unit (version regex) | Release safety |
| AC-PLT-012 | 1 BDD | 1 BDD + 1 unit (clean tree check) | Release gate |
| AC-TPL-GRAPH-SELFTEST | 1 BDD | Already 2/2 ✓ | Graph health |

**Timeline:** Complete by next release (v3.4.0)

### Phase 2: DevEx Foundation (Priority 2)
Target: Core developer commands (doctor, check, help-flows)

| AC ID | Current Coverage | Target Coverage | Rationale |
|-------|------------------|-----------------|-----------|
| AC-PLT-001 | 1 BDD | 2 BDD (happy + missing tool) | Onboarding critical |
| AC-PLT-002 | 1 BDD | 1 BDD + 1 unit (YAML parsing) | Flow discovery |
| AC-PLT-003 | 1 BDD | 2 BDD (pass + clippy fail) | Fast feedback loop |

**Timeline:** Complete by v3.5.0

### Phase 3: Agent Interface (Priority 3)
Target: Skills tooling and platform APIs

| AC ID | Current Coverage | Target Coverage | Rationale |
|-------|------------------|-----------------|-----------|
| AC-TPL-SKILLS-FMT | 1 BDD | 1 BDD + 1 unit (frontmatter validation) | Agent reliability |
| AC-TPL-SKILLS-LINT | 1 BDD | 1 BDD + 1 unit (link validation) | Agent reliability |
| AC-TPL-SKILLS-ALIGN-001 | 1 BDD | 2 BDD (alignment check + drift detection) | Agent correctness |

**Timeline:** Complete by v3.6.0

### Phase 4: Platform Completeness (Priority 4)
Target: Remaining kernel ACs

All remaining kernel ACs with 1/1 coverage should be evaluated for test diversity opportunities.

**Timeline:** Ongoing, as part of normal feature work

---

## 5. Distinction: Kernel Now vs Template Future

### Kernel (Must Implement Now)
These are enforced by `cargo xtask selftest` step 8 (AC coverage check):
- All ACs with `must_have_ac: true` and `tags: [kernel]`
- Currently: 53 kernel ACs, 35 passing, 27 failing
- **Selftest will fail** until all kernel ACs pass

### Template Future (Informational)
These are documented but NOT enforced by selftest:
- All ACs with `must_have_ac: false` or `tags: [future]`
- Currently: 11 future ACs
- Examples: CLI task commands, authentication modes, suggest-next CLI
- **Selftest ignores** these ACs

### How to Identify
1. Look at `specs/spec_ledger.yaml` for the AC
2. Check `must_have_ac` field (true = kernel, false = future)
3. Check `tags` array (kernel = kernel, future = informational)

---

## 6. UNKNOWN ACs (None Currently)

**Current State:** No UNKNOWN ACs detected by `ac-status`.

All 65 ACs in `spec_ledger.yaml` have corresponding test mappings (BDD or unit tests), even if those tests are currently failing.

**If UNKNOWN ACs appear in the future:**
- Document in this section why they're unknown:
  - Template placeholder not yet adopted?
  - Test infrastructure missing?
  - Intentionally deferred pending design decision?
- Link to relevant ADR or GitHub issue
- Provide timeline for resolution

---

## 7. Action Items

### Immediate (This Sprint)
1. ✅ **Complete Task 1:** Normalize ACs and document test hygiene (this file)
2. **Next:** Implement failing kernel ACs starting with foundation commands
3. **Track:** Update this file when AC status changes

### Short-Term (Next 2 Sprints)
1. Achieve selftest-green by implementing all 27 failing kernel ACs
2. Add second test scenarios for Phase 1 high-priority ACs (security, release, graph)
3. Update TEMPLATE-CONTRACTS.md with test diversity roadmap reference

### Long-Term (v3.6.0+)
1. Establish 2-3 test scenarios per kernel AC as standard practice
2. Complete Phase 2-4 test diversity improvements
3. Review and archive or promote future ACs based on actual service needs

---

## 8. References

- **Spec Ledger:** `specs/spec_ledger.yaml` (source of truth for all ACs)
- **Feature Status:** `docs/feature_status.md` (auto-generated AC test status)
- **Template Contracts:** `docs/explanation/TEMPLATE-CONTRACTS.md` (kernel vs customization)
- **AC Status Command:** `cargo xtask ac-status` (regenerates feature_status.md)
- **Selftest:** `cargo xtask selftest` (validates kernel contracts)

---

## Changelog

- **2025-11-26:** Initial comprehensive AC normalization and test hygiene documentation
  - Classified 27 failing kernel ACs vs 4 failing future ACs
  - Documented test diversity gap (59/65 ACs have single-test coverage)
  - Created 4-phase test diversity roadmap
  - Clarified kernel vs template future distinction
