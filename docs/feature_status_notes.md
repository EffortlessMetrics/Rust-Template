# Feature Status Notes

**Last Updated:** 2025-12-01
**Template Version:** v3.3.5
**Purpose:** Document AC coverage state, @ci-only testing pattern, and meta/CI-only contracts.

---

## Executive Summary

As of v3.3.5 (December 2025 update):
- **Total ACs:** 105
- **Kernel ACs (must_have_ac: true):** 61 (all passing)
- **Template ACs (must_have_ac: false):** 44
  - **Passing:** 27
  - **Unknown:** 17 (intentionally soft)

**Status:** The template is at "LLM-native Rust cell 1.0" state. All kernel ACs pass. The 17 remaining UNKNOWN ACs are intentionally soft contracts (`must_have_ac: false`) covering governance documentation, guidance policies, and meta/CI-only contracts that aren't exercised in local selftest.

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

### Template/Meta ACs: 17 UNKNOWN (Intentionally Soft)

17 ACs remain UNKNOWN in `feature_status.md`. This is **intentional** – they're governed via documentation, lint warnings, CI workflows, or manual review rather than hard test gates.

#### Skills Governance (7 ACs)

| AC ID | Type | Why UNKNOWN |
|-------|------|-------------|
| AC-TPL-SKILLS-GOVERNANCE-001 | Doc exists | `docs/SKILLS_GOVERNANCE.md` exists. Validated by file presence. |
| AC-TPL-SKILLS-GOVERNANCE-002 | Guidance | Skills have REQ/AC coverage. Manual code review validation. |
| AC-TPL-SKILLS-GOVERNANCE-003 | Doc exists | `docs/SKILLS_TEMPLATE.md` exists. Validated by file presence. |
| AC-TPL-SKILLS-DESCRIPTION-QUALITY | Guidance | skills-lint warns on low-quality descriptions. Not a hard gate. |
| AC-TPL-SKILLS-ALLOWED-TOOLS-SAFETY | Guidance | skills-lint warns on tool grants. Least-privilege is advisory. |
| AC-TPL-SKILLS-FLOW-MAPPING | Guidance | skills-lint warns on anti-patterns. Not a hard gate. |
| AC-TPL-SKILLS-LIFECYCLE-DOCS | Doc exists | Lifecycle documented in SKILLS_GOVERNANCE.md. |

#### Agents Governance (6 ACs)

| AC ID | Type | Why UNKNOWN |
|-------|------|-------------|
| AC-TPL-AGENTS-DESCRIPTION-QUALITY | Guidance | agents-lint warns on low-quality descriptions. Not a hard gate. |
| AC-TPL-AGENTS-TOOLS-PERMISSION-SAFETY | Guidance | agents-lint warns on tool grants. Not a hard gate. |
| AC-TPL-AGENTS-MODEL-POLICY | Guidance | agents-lint warns on expensive models. Not a hard gate. |
| AC-TPL-AGENTS-SKILLS-REFERENCES | Validation | agents-lint errors on missing skills. Enforced but not BDD-tested. |
| AC-TPL-AGENTS-LIFECYCLE-DOCS | Doc exists | Lifecycle documented in AGENTS_GOVERNANCE.md. |

#### Platform/Template Infrastructure (5 ACs)

| AC ID | Type | Why UNKNOWN |
|-------|------|-------------|
| AC-PLT-AC-DEMOTION-GOVERNED | Policy | AC demotion is a governance policy, not a test gate. See §6. |
| AC-TPL-BDD-EXIT-CODES | Harness | Tests the test harness itself. CI validates `[BDD-PASS]` output. |
| AC-TPL-BUNDLE-MINIMAL-SCOPE | Guidance | Bundle scope is reviewed manually, not automatically gated. |
| AC-TPL-EXAMPLE-FORK-BUILDS | Example | CI job validates example fork. Not tested locally. |
| AC-TPL-XTASK-SPEC-ROOT | Testing infra | SPEC_ROOT behavior is implicitly tested. Unit test exists but not mapped. |

These are correctly showing as UNKNOWN because:
- They're documentation, guidance, or governance policies (not testable as BDD)
- They're validated via lint warnings, CI workflows, or manual review
- They're intentionally `must_have_ac: false` to allow fork customization

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

## 5. AC Type Classification

This section provides the canonical reference for understanding the three types of acceptance criteria and how to work with them.

### Classification Overview

| Type | `must_have_ac` | Enforcement | Testing | Example | Fork Behavior |
|------|---|---|---|---|---|
| **Kernel** | `true` | Selftest fails if not passing | Tested locally via BDD/unit | AC-PLT-001 (doctor) | Cannot be demoted without breaking compatibility |
| **Template** | `false` | Documented but not enforced | Tested locally when enabled | AC-TPL-GOV-FRICTION | Can be demoted/customized freely |
| **Meta/CI-only** | `false` | Not tested locally, validated in CI | CI-only scenarios or harness-level tests | AC-TPL-BDD-EXIT-CODES | Test infrastructure, not service behavior |

### Detailed Type Descriptions

#### Kernel ACs (`must_have_ac: true`)

Kernel ACs are **service-level contracts** that every fork must maintain. They define the minimum viable governance layer, DevEx commands, and platform guarantees.

**Characteristics:**
- Enforced by `cargo xtask selftest` – the step-8 AC coverage gate fails if any kernel AC is failing
- Tested in local development (both BDD and unit tests)
- Cannot be demoted in forks without breaking forward compatibility
- Include: commands (doctor, check, selftest, ci-local), platform APIs (/status, /graph), and governance guarantees

**Examples:**
- AC-PLT-001: `cargo xtask doctor` validates environment
- AC-PLT-015: Selftest enforces devex contract (required commands exist)
- AC-TPL-CONFIG-VALIDATION: Configuration schema validation
- AC-TPL-GRAPH-SELFTEST: Graph invariants enforcement

#### Template ACs (`must_have_ac: false`)

Template ACs are **customization points** or **optional features** that the template provides but forks can customize or disable. They're not enforced by selftest.

**Characteristics:**
- Documented in `spec_ledger.yaml` but **not enforced by selftest**
- Tested locally when tests are present
- Can be demoted (changed to `must_have_ac: false` per AC) without breaking the template
- Include: governance artifacts (friction logging, fork registry), advanced platform features, and example workflows

**Examples:**
- AC-TPL-GOV-FRICTION: Friction logging infrastructure
- AC-TPL-FORKS-STATUS-SUMMARY: Fork registry and visibility
- AC-TPL-PLATFORM-AUTH-BASIC: Basic auth mode for platform APIs
- AC-TPL-OVERRIDE-DOC: Documentation for customizing kernel ACs

**Note:** Many template ACs are currently showing as UNKNOWN because they're **future work** – they're documented as goals but not yet implemented with tests. This is not a bug; it's the natural state of a roadmap.

#### Meta / CI-only ACs (`must_have_ac: false`, tagged `[harness]` or `[example]`)

Meta ACs describe **test harness behaviour** or **example workspace validation**, not service-level contracts. They're intentionally not tested in local selftest.

**Characteristics:**
- Not tested in local `cargo xtask selftest` (excluded via `@ci-only` tag in BDD)
- Validated only in CI environments
- No entry in the "Unmapped ACs (Service Behaviour)" section (appears in "Meta / CI-only ACs" section instead)
- Include: test harness semantics, example fork builds, and recursive selftest validation

**Examples:**
- AC-TPL-BDD-EXIT-CODES: Harness returns exit 0 when tests pass (validated by CI harness)
- AC-TPL-EXAMPLE-FORK-BUILDS: Example workspace builds and passes selftest in CI

### How to Identify

To determine an AC's type, check `specs/spec_ledger.yaml`:

1. **Look at `must_have_ac` field:**
   - `true` → Kernel AC
   - `false` → Template or Meta AC

2. **Check the `tags` array:**
   - `[kernel]` → Kernel AC (always kernel)
   - `[template]` → Template AC (unless also has `[harness]` or `[example]`)
   - `[template, harness]` → Meta AC (harness-level)
   - `[template, example]` → Meta AC (example workspace)

3. **Check the `tests` section:**
   - `type: ci` → Meta AC (CI-only test)
   - `type: bdd` or `type: unit` → Kernel or Template AC

### Working with Each Type

#### Kernel AC: Add/Change

To add or modify a kernel AC:

```bash
# 1. Edit spec_ledger.yaml, add AC with must_have_ac: true, tags: [kernel, ...]
# 2. Add BDD scenario in specs/features/*.feature with @AC-XXX tag
# 3. Optionally add unit tests in crates/*/src/ or crates/*/tests/
# 4. Run the validation ladder:
cargo xtask check
cargo xtask test-changed
cargo xtask test-ac AC-XXX
cargo xtask selftest  # Must pass
```

#### Template AC: Add/Change

To add or modify a template AC:

```bash
# 1. Edit spec_ledger.yaml, add AC with must_have_ac: false, tags: [template, ...]
# 2. Add BDD scenario and/or unit tests
# 3. Run local tests:
cargo xtask check
cargo xtask test-changed
# Selftest is not a gate for template ACs, but selftest should still pass overall
cargo xtask selftest
```

#### Meta/CI-only AC: Add

To add a meta AC (rare):

```bash
# 1. Edit spec_ledger.yaml, add AC with:
#    must_have_ac: false
#    tags: [template, harness] or [template, example]
#    tests: [{ type: ci, ... }]
# 2. Add @ci-only scenarios in BDD, or describe the CI-level validation
# 3. Document why it's meta in the AC text
# 4. Local selftest will skip it (correct); CI will validate it
```

---

## 6. Changing AC classification (kernel / template / meta)

Changing an AC's type is a **kernel contract change**, not just a test tweak.

**Rules:**

- Any change to `must_have_ac` must be treated as a kernel version change.
- Demoting a kernel AC (`must_have_ac: true` → `false`) requires:
  - An ADR explaining the decision and impact
  - A kernel minor version bump
  - Updates to `docs/KERNEL_SNAPSHOT.md` and `docs/feature_status.md` explaining the change
- Promoting a template AC to kernel (`false` → `true`) requires:
  - Confirmed test coverage (unit and/or BDD)
  - Inclusion in the kernel contract JSON emitted by `cargo xtask release-bundle`
- Meta/CI-only ACs (e.g. tagged `["template", "harness"]` or validated via `type: ci` tests)
  should not be promoted to kernel; they verify the harness, not service behaviour.

**Anti-patterns:**

- Flipping `must_have_ac` just to get CI green, without ADRs and versioning.
- Hiding fragile behaviour behind `template`/`meta` tags instead of fixing it.

For the step-by-step workflow, see `docs/how-to/change-acceptance-criterion.md`.

---

## 7. References

- **Spec Ledger:** `specs/spec_ledger.yaml` (source of truth for all ACs)
- **Feature Status:** `docs/feature_status.md` (auto-generated AC test status)
- **Template Contracts:** `docs/explanation/TEMPLATE-CONTRACTS.md` (kernel vs customization)
- **AC Status Command:** `cargo xtask ac-status` (regenerates feature_status.md)
- **Selftest:** `cargo xtask selftest` (validates kernel contracts)
- **BDD Implementation:** `crates/xtask/src/commands/bdd.rs` (@ci-only filtering)

---

## Changelog

- **2025-12-01:** Slice B – Harness & SPEC_ROOT ACs
  - Mapped unit tests for AC-TPL-BDD-EXIT-CODES (`is_bdd_success` tests in bdd.rs)
  - Mapped unit test for AC-TPL-XTASK-SPEC-ROOT (`spec_root_resolved` in tasks.rs)
  - Template ACs: 29 passing, 15 unknown (reduced from 17 unknown)
- **2025-12-01:** v3.3.4 AC coverage expansion
  - Total ACs increased from 81 to 105 (Skills/Agents governance ACs added)
  - Added BDD tests for AC-TPL-SKILLS-NAME-FORMAT, AC-TPL-AGENTS-NAME-FORMAT, AC-PLT-DOC-INDEX-FRONTMATTER
  - Updated soft AC documentation: 17 intentionally soft ACs categorized by purpose
  - Skills governance ACs documented (7 ACs)
  - Agents governance ACs documented (5 ACs)
  - Platform/template infrastructure ACs documented (5 ACs)
- **2025-11-27:** Track B + C refinement
  - Added section 6: AC classification governance rules
  - Documented rules for changing `must_have_ac` and versioning implications
  - Added anti-patterns section
- **2025-11-27:** Updated for v3.3.3 final state
  - All kernel and template ACs passing
  - Documented @ci-only testing pattern
  - Clarified meta/CI-only ACs (AC-TPL-BDD-EXIT-CODES, AC-TPL-EXAMPLE-FORK-BUILDS)
  - Added guidance for zero UNKNOWN preference
- **2025-11-26:** Initial comprehensive AC normalization and test hygiene documentation
