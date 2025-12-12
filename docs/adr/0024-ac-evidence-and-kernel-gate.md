---
id: ADR-0024
title: AC Evidence Model and Kernel Coverage Gate
doc_type: adr
status: accepted
date: 2025-12-11
authors: AI-Assisted Implementation
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-AC-GOVERNANCE, REQ-PLT-DEVEX-CONTRACT]
acs: [AC-PLT-AC-GOVERNANCE, AC-TPL-SELFTEST-GATE]
adrs: [ADR-0003, ADR-0005, ADR-0023]
---

# ADR-0024: AC Evidence Model and Kernel Coverage Gate

## Context

The AC governance system needs a formal, documented model for how AC status is determined from test evidence. The current implementation has several gaps:

1. **Implicit rules**: Status classification rules are embedded in code across multiple files (`ac_status.rs`, `selftest.rs`, `coverage.rs`) without formal documentation.

2. **Unit test evidence**: Unit tests mapped in `spec_ledger.yaml` are "assumed to run" because `cargo xtask check` executes them, but this assumption is implicit and not verified.

3. **No unified evidence model**: BDD coverage and unit test mappings are handled separately, making it hard to reason about overall AC coverage.

4. **Undocumented gate semantics**: The `KERNEL_UNKNOWN_BUDGET` and `XTASK_STRICT_AC_COVERAGE` environment variables were implemented but lacked ADR documentation.

Additionally, the relationship between:
- Spec mappings (what tests *should* exist)
- Runtime evidence (what tests *actually* ran)
- AC status (the derived governance verdict)

...was not formally defined.

## Decision

### 1. Evidence Model

Introduce an `AcEvidence` struct that captures all evidence for an AC:

```rust
pub struct AcEvidence {
    pub ac_id: String,
    pub is_kernel: bool,

    // Spec-derived (from spec_ledger.yaml tests: array)
    pub unit_mapped: usize,   // Count of type: unit tests
    pub bdd_mapped: usize,    // Count of type: bdd/integration tests

    // Runtime (from coverage.jsonl)
    pub bdd_passed: usize,    // BDD scenarios that passed
    pub bdd_failed: usize,    // BDD scenarios that failed
}
```

### 2. Status Classification Rules

The `AcEvidence::status()` method computes AC status using these rules:

| Condition | Status | Rationale |
|-----------|--------|-----------|
| `bdd_failed > 0` | **FAIL** | Any failing BDD scenario fails the AC |
| `bdd_passed > 0 OR unit_mapped > 0` | **PASS** | Evidence of coverage exists |
| Otherwise | **UNKNOWN** | No test evidence |

**Rationale for unit_mapped as evidence:**

Unit tests are "presumed to run" because:
1. `cargo xtask selftest` runs `cargo xtask check` which executes `cargo test --workspace`
2. If a unit test is mapped in `spec_ledger.yaml`, it will be executed
3. If the unit test fails, `selftest` fails before reaching the AC coverage check

Therefore, the existence of a unit test mapping is sufficient evidence that the AC has coverage, even without explicit runtime verification.

### 3. Kernel AC Definition

An AC is a **kernel AC** if and only if `must_have_ac` is `true` at both levels:
- Requirement level: `req.must_have_ac == true`
- AC level: `ac.must_have_ac == true`

This uses AND semantics: `effective_must_have = req.must_have_ac && ac.must_have_ac`

### 4. Kernel Coverage Gate

The selftest AC coverage check enforces:

| Condition | Result |
|-----------|--------|
| Any kernel AC is FAIL | **Selftest fails** (hard gate) |
| Kernel UNKNOWN count > budget | **Selftest fails** (soft gate) |
| Non-kernel FAIL/UNKNOWN | **Selftest warns** (informational) |

### 5. Unknown Budget Configuration

| Environment Variable | Effect |
|---------------------|--------|
| `XTASK_STRICT_AC_COVERAGE=1` | Equivalent to budget = 0 |
| `KERNEL_UNKNOWN_BUDGET=N` | Allow at most N unknown kernel ACs |
| Neither set | Unlimited (backward compatible) |

When both are set, `XTASK_STRICT_AC_COVERAGE=1` takes precedence (budget forced to 0).

### 6. Ratchet Mechanism

To prevent coverage regression:

1. CI sets `KERNEL_UNKNOWN_BUDGET` to the current unknown count on main
2. As tests are added, the count decreases
3. The budget is lowered to match (monotonic decrease)
4. Eventually, `XTASK_STRICT_AC_COVERAGE=1` is enabled (budget = 0)

Example progression:
```yaml
# Week 1: 50 unknown kernel ACs
KERNEL_UNKNOWN_BUDGET: "50"

# Week 2: Added 10 tests, now 40 unknown
KERNEL_UNKNOWN_BUDGET: "40"

# Final state: All kernel ACs have coverage
XTASK_STRICT_AC_COVERAGE: "1"
```

## Consequences

### Positive

1. **Formal semantics**: The evidence model provides clear, documented rules for AC status computation.

2. **Unit test recognition**: Unit tests mapped in spec are now recognized as valid evidence, reducing false UNKNOWN counts.

3. **Gradual enforcement**: The budget mechanism allows incremental improvement without blocking all development.

4. **Audit trail**: The `AcEvidence` struct captures both spec intent (mapped) and runtime reality (executed).

5. **Consistency**: All AC status computations use the same rules, whether in `ac-status`, `selftest`, or other tools.

### Negative

1. **Implicit unit test verification**: We assume unit tests run if mapped, without explicit verification. A mapped but deleted test would show as PASS. (Mitigated: test failures would break `cargo test`.)

2. **Budget maintenance overhead**: CI needs to manually lower the budget as coverage improves. (Mitigated: This is intentional friction to ensure coverage improvements are locked in.)

### Neutral

1. **Schema version unchanged**: The `AcEvidence` struct is internal; the JSON output schema (`ac-status --json`) remains at v2.0.

2. **Backward compatible defaults**: Unlimited budget preserves existing behavior for repos that haven't adopted the gate.

## Compliance

### Automated Enforcement

- **Selftest step 10**: Enforces kernel coverage gate with budget/strict configuration
- **Pre-commit hook**: Regenerates `docs/feature_status.md` which shows AC status
- **CI workflow**: Sets `KERNEL_UNKNOWN_BUDGET` to enforce coverage baseline

### Manual Review

- **ADR changes**: Modifications to this ADR require review
- **Budget changes**: Lowering `KERNEL_UNKNOWN_BUDGET` in CI requires PR review
- **AC demotion**: Setting `must_have_ac: false` on a kernel AC requires justification

## Notes

### Evidence Sources

| Source | Evidence Type | When Available |
|--------|--------------|----------------|
| `spec_ledger.yaml` | Spec mappings (unit_mapped, bdd_mapped) | Always (if AC exists) |
| `target/ac/coverage.jsonl` | BDD runtime results | After BDD tests run |
| `cargo test` | Unit test execution | Implicit in selftest |

### Related ADRs

- **ADR-0003**: Establishes spec and BDD as source of truth
- **ADR-0005**: Defines selftest as the single governance gate
- **ADR-0023**: Defines coverage.jsonl as primary BDD truth source

### Future Considerations

1. **Explicit unit test verification**: Could parse `cargo test` output to verify mapped tests actually ran. Not implemented due to complexity.

2. **Per-AC evidence audit**: Could store detailed evidence (which tests ran, when) for compliance reporting.

3. **Coverage quality metrics**: Could weight different test types (BDD > unit for behavioral coverage).

---

## Milestone: Zero Kernel Unknowns (2025-12-11)

On 2025-12-11, this ADR's goals were fully achieved:

- **Kernel Unknowns: 13 → 0** — All 72 kernel ACs now have test evidence (unit or BDD)
- **Budget set to 0** — `KERNEL_UNKNOWN_BUDGET=0` in tier1-selftest.yml
- **Strict mode enabled on main** — `XTASK_STRICT_AC_COVERAGE=1` enforced on main branch
- **Ratchet locked** — Budget can only stay at 0 or be increased with ADR justification

### Tests Added to Reach Zero

The following unit tests were mapped to eliminate the final 13 unknowns:

| AC ID | Test(s) Mapped |
|-------|---------------|
| AC-TPL-001 | `test_health_returns_ok` |
| AC-TPL-002 | `test_version_returns_build_info` |
| AC-PLT-007 | `test_audit_recovery_has_four_steps` |
| AC-PLT-009 | `test_docs_check_validates_eight_consumers` |
| AC-PLT-017 | `test_status_command_exists`, `test_status_metrics_categories`, `test_count_governance_returns_counts` |
| AC-PLT-018 | `test_dev_up_command_exists`, `test_dev_up_required_steps`, `test_dev_up_provides_next_steps` |
| AC-TPL-CLI-JSON-OUTPUT | `test_cli_json_output_contract`, `version_json_shape_is_stable` |
| AC-TPL-XTASK-NONINTERACTIVE | `test_ci_detection_via_ci_var`, `test_noninteractive_mode_contract`, `test_environment_variables_documented` |
| AC-TPL-SKILLS-GUIDE-001 | `test_skills_guide_doc_exists` |
| AC-TPL-SKILLS-ALIGN-001 | `test_skills_align_with_workflows` |
| AC-TPL-VERSION-MANIFEST | `test_manifest_load_from_repo` |
| AC-TPL-VERSION-DRYRUN | `test_plan_changes_with_temp_files`, `test_apply_changes_dry_run` |
| AC-TPL-VERSION-ATOMIC | `test_apply_changes_actual`, `test_apply_changes_atomicity_on_mismatch` |

### Guardrails Established

Going forward, any new kernel AC (`must_have_ac: true`) **must** land with at least one mapped test, or:
1. Go through the AC demotion governance path (set `must_have_ac: false` with ADR justification)
2. Temporarily increase `KERNEL_UNKNOWN_BUDGET` with review approval (not recommended)
