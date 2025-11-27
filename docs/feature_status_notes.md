# Feature Status Notes

**Last Updated:** 2025-11-27
**Template Version:** v3.3.3
**Purpose:** Document AC coverage state, @ci-only testing pattern, and meta/CI-only contracts.

---

## Executive Summary

As of v3.3.3:
- **Total ACs:** 81
- **Kernel ACs (must_have_ac: true):** 52
- **Template ACs (must_have_ac: false):** 29
- **Currently Passing:** 79 ACs
- **Meta/CI-only (UNKNOWN):** 2 ACs (by design)

**Status:** The template is at "LLM-native Rust cell 1.0" state. All kernel and service behaviour ACs pass. The two remaining UNKNOWN ACs are intentionally meta/CI-only contracts that aren't exercised in local selftest.

---

## 1. AC Coverage Summary

### Kernel ACs: All Passing

All 52 kernel ACs (`must_have_ac: true`) are passing:
- DevEx commands (doctor, check, selftest, ci-local)
- Platform APIs (/status, /graph, /docs, governance endpoints)
- UI, graph invariants, config validation, auth
- Release management, skills tooling, task lifecycle

### Template ACs: All Passing

All template behaviour ACs are passing where implemented.

### Meta/CI-only ACs: 2 UNKNOWN (By Design)

Two ACs remain UNKNOWN in `feature_status.md`. This is **intentional**:

| AC ID | Type | Why UNKNOWN |
|-------|------|-------------|
| AC-TPL-BDD-EXIT-CODES | Harness contract | Tests the test harness itself, not service behaviour. Validated at CI/harness level. |
| AC-TPL-EXAMPLE-FORK-BUILDS | Example fork | Tests a nested example workspace. CI-only validation avoids coupling template to demo. |

These are correctly showing as UNKNOWN because:
- They're not service contracts
- No test that `ac-status` scans is mapped to them (by design)
- They're validated in CI, not local selftest

**If you want zero UNKNOWN rows:** See §3 for options.

---

## 2. The @ci-only Testing Pattern

Some BDD scenarios are tagged `@ci-only` to exclude them from local development runs. This pattern is used for:

1. **Recursive scenarios** - Tests that run `selftest` from within selftest
2. **Git worktree scenarios** - Tests that create temporary worktrees (can flake with VS Code Git extension)
3. **Heavy integration tests** - Tests that spawn processes or access external resources

### How It Works

In `crates/xtask/src/commands/bdd.rs`:

```rust
// When not in CI, automatically exclude @ci-only scenarios
if std::env::var("CUCUMBER_TAG_EXPRESSION").is_err() && !in_ci {
    cmd.env("CUCUMBER_TAG_EXPRESSION", "not @ci-only");
    println!("ℹ Excluding @ci-only scenarios from local run");
}
```

### Current @ci-only Scenarios

Scenarios marked `@ci-only` in `specs/features/xtask_devex.feature`:
- `test-changed builds tag expression for changed features` - Git worktree operations
- `selftest enforces devex contract` - Recursive selftest validation
- `selftest displays condensed summary with 8 steps` - Recursive selftest
- `selftest summary shows all step names` - Recursive selftest
- `selftest summary shows pass/fail status for each step` - Recursive selftest
- `selftest shows actionable error messages on failure` - Recursive selftest
- `selftest respects XTASK_LOW_RESOURCES environment variable` - Recursive selftest
- `selftest runs non-interactively with XTASK_NONINTERACTIVE=1` - Recursive selftest

### When to Use @ci-only

Tag a scenario `@ci-only` when:
- It runs `cargo xtask selftest` from within BDD (recursive execution)
- It creates git worktrees or modifies `.git` state
- It depends on CI-specific environment (clean checkout, no VS Code)
- It's inherently slow or resource-intensive

**Important:** Always ensure the AC has unit test coverage or stable BDD scenarios for local validation. @ci-only should supplement, not replace, local testing.

---

## 3. Options for Zero UNKNOWN ACs

If you prefer `feature_status.md` to have zero UNKNOWN rows, you have two options:

### Option A: Move Meta ACs to Separate Documentation

Move `AC-TPL-BDD-EXIT-CODES` and `AC-TPL-EXAMPLE-FORK-BUILDS` to a separate doc (e.g., `docs/TEST_HARNESS_CONTRACTS.md`) and remove them from `spec_ledger.yaml`.

Result:
- `feature_status.md` becomes purely about service behaviours
- Harness and example contracts are still documented, just not in the AC table

### Option B: Keep Them with Clear Annotations

Keep them in the ledger but update the Unmapped ACs section in `feature_status.md` to split:

```markdown
## Unmapped ACs (Service-Level)
*(List should be empty in this repo.)*

## Meta/CI-only ACs (Not Executed Locally)
- AC-TPL-BDD-EXIT-CODES - Harness semantics, verified in CI harness output
- AC-TPL-EXAMPLE-FORK-BUILDS - Example workspace, verified by CI job
```

**Recommendation:** The current state (Option B implicitly) is honest and accurate. The UNKNOWN status correctly indicates "not tested in local selftest" rather than "broken."

---

## 4. Test Diversity

Most ACs have good coverage:
- BDD scenarios for behaviour validation
- Unit tests for implementation correctness
- CI-only scenarios for integration/recursive testing

### ACs with Multiple Test Types (Examples)
- AC-TPL-CONFIG-VALIDATION: 2 tests (BDD + unit)
- AC-TPL-GRAPH-MERMAID: 2 tests (BDD + unit)
- AC-TPL-LOG-NO-SECRETS: 2 tests (BDD + unit)
- AC-TPL-TASK-TRANSITIONS: 2 tests (unit: allowed + forbidden)
- AC-TPL-GRAPH-SELFTEST: 4 tests (3 unit + 1 BDD @ci-only)
- AC-PLT-015: 4 tests (3 unit + 1 BDD @ci-only)

---

## 5. Distinction: Kernel vs Template vs Meta

| Type | Enforcement | Example |
|------|-------------|---------|
| **Kernel** | `must_have_ac: true` - Selftest fails if not passing | AC-PLT-001 (doctor) |
| **Template** | `must_have_ac: false` - Documented but not enforced | AC-TPL-GOV-FRICTION |
| **Meta/CI-only** | Not tested locally, validated in CI | AC-TPL-BDD-EXIT-CODES |

### How to Identify

1. Check `specs/spec_ledger.yaml` for the AC
2. Look at `must_have_ac` field (true = kernel, false = template/meta)
3. Check `tags` array (kernel, template, harness, example, ci-only)
4. Check `tests:` section for `type: ci` entries

---

## 6. References

- **Spec Ledger:** `specs/spec_ledger.yaml` (source of truth for all ACs)
- **Feature Status:** `docs/feature_status.md` (auto-generated AC test status)
- **Template Contracts:** `docs/explanation/TEMPLATE-CONTRACTS.md` (kernel vs customization)
- **AC Status Command:** `cargo xtask ac-status` (regenerates feature_status.md)
- **Selftest:** `cargo xtask selftest` (validates kernel contracts)
- **BDD Implementation:** `crates/xtask/src/commands/bdd.rs` (@ci-only filtering)

---

## Changelog

- **2025-11-27:** Updated for v3.3.3 final state
  - All kernel and template ACs passing
  - Documented @ci-only testing pattern
  - Clarified meta/CI-only ACs (AC-TPL-BDD-EXIT-CODES, AC-TPL-EXAMPLE-FORK-BUILDS)
  - Added guidance for zero UNKNOWN preference
- **2025-11-26:** Initial comprehensive AC normalization and test hygiene documentation
