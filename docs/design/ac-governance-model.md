---
id: DESIGN-PLT-AC-GOVERNANCE-MODEL-001
title: AC Governance Model
author: governance-system
doc_type: design_doc
date: 2025-12-07
status: published
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-AC-GOVERNANCE]
tags: [platform, governance, ac-coverage]
acs: [AC-PLT-AC-DEMOTION-GOVERNED]
adrs: [ADR-0005, ADR-0023]
---

# AC Governance Model

This document defines the Acceptance Criterion (AC) governance model for the Rust-as-Spec platform cell. It consolidates the classification, data flow, and enforcement rules into a single reference.

## 1. Classification

### 1.1 must_have_ac Flag

Each AC has an effective `must_have_ac` value computed as the AND of the requirement and AC flags:

| REQ must_have_ac | AC must_have_ac | Effective | Classification |
|------------------|-----------------|-----------|----------------|
| true             | true            | **true**  | Kernel AC      |
| true             | false           | false     | Optional AC    |
| false            | true            | false     | Optional AC    |
| false            | false           | false     | Optional AC    |

**Kernel ACs** (`must_have_ac=true`):
- Required for template health
- Enforced by selftest + CI SLO gates
- Must have BDD scenario coverage or equivalent test mapping
- Blocking: unknown or failing kernel AC = CI failure

**Optional ACs** (`must_have_ac=false`):
- Informational or best-effort
- Not gating for selftest/CI
- May have tests via CI workflows, governance lint, or documentation
- Useful for tracking ergonomics, future plans, or guidance

### 1.2 Status Values

| Status    | Definition |
|-----------|------------|
| `pass`    | All mapped tests passed |
| `fail`    | At least one mapped test failed |
| `unknown` | No tests mapped, or all mapped tests were skipped/missing |

### 1.3 Test Types

ACs can be validated by different test types:

| Type        | Source                    | Discovery |
|-------------|---------------------------|-----------|
| `bdd`       | BDD scenarios with @AC-ID tag | coverage.jsonl |
| `unit`      | Rust unit tests with AC marker | test output parsing |
| `integration` | Integration tests | coverage.jsonl |
| `ci`        | CI workflow validation | Not tracked in coverage.jsonl |
| `docs`      | Documentation existence check | Manual review |
| `manual`    | Manual review process | Manual review |

## 2. Data Flow

```
specs/spec_ledger.yaml (REQs + ACs + must_have_ac)
  │
  ├─→ specs/features/*.feature (@AC-… tags on scenarios)
  │
  ├─→ tests (BDD + unit) with AcCoverageWriter
  │     ↓
  │   target/ac/coverage.jsonl (scenario → AC mapping)
  │
  ↓
ac_parsing::{parse_ledger_with_metadata, parse_ac_coverage}
  │
  ├─→ cargo xtask ac-status (--summary / --json)
  │     Schema: v2.0
  │     ├─ ac-coverage (--todo [--must-have])
  │     ├─ ac-report (text / markdown / html / json)
  │     ├─ ac-history (snapshots)
  │     └─ ac-slo (SLOs over snapshots)
  │
  └─→ selftest step 10 + XTASK_STRICT_AC_COVERAGE + CI SLO gates
```

### 2.1 Primary Source: coverage.jsonl

The coverage JSONL file (`target/ac/coverage.jsonl`) is the primary source for AC coverage:

- Written by `AcCoverageWriter` during BDD test execution
- Each line is a JSON record with: `ac_id`, `status`, `feature`, `scenario`, `tags`
- Streaming format: resilient to cucumber's exit() behavior
- Scenario identity: keyed by `feature::scenario_name` to avoid collisions

### 2.2 Skipped Scenario Semantics

Skipped scenarios (status = "skipped") are excluded from pass/fail aggregation:
- AC with only "passed" scenarios → Pass
- AC with any "failed" scenarios → Fail
- AC with only "skipped" scenarios → Unknown

## 3. CI Gates

### 3.1 selftest (cargo xtask selftest)

Step 10 validates AC coverage:
- Counts kernel ACs with status = unknown
- Reports coverage percentage

With `XTASK_STRICT_AC_COVERAGE=1`:
- No unknown must_have_ac ACs allowed
- Any unknown kernel AC = selftest failure

### 3.2 ac-slo (cargo xtask ac-slo)

SLO enforcement over snapshot history:
- Coverage threshold (e.g., ≥80%)
- Kernel blocker count (e.g., 0 failing/unknown kernel ACs)
- Trend analysis (no regression from previous snapshots)

### 3.3 CI Configuration

In `specs/required_checks.yaml`:
```yaml
required_checks:
  - tier1-selftest  # Runs selftest with strict AC coverage
```

On `main` / `release/*` branches:
- `XTASK_STRICT_AC_COVERAGE=1` enabled
- SLO gate via `ac-slo` command

### 3.4 SLOs & Gates (Policy)

The following SLOs are enforced on `main` branch in `.github/workflows/tier1-selftest.yml`:

**Selftest (strict mode):**
```yaml
env:
  XTASK_STRICT_AC_COVERAGE: ${{ github.ref == 'refs/heads/main' && '1' || '' }}
run: nix develop -c cargo xtask selftest
```
- Fails if any `must_have_ac=true` AC has status `unknown`
- All kernel ACs must be covered by tests

**SLO gate (kernel):**
```bash
cargo xtask ac-slo \
  --dir artifacts/ac-status \
  --min-coverage 80.0 \
  --max-blockers 0
```
- `--min-coverage 80.0`: Overall AC coverage must be ≥80%
- `--max-blockers 0`: No kernel (`must_have_ac=true`) ACs may be failing or unknown
- Only applies to snapshots in the artifacts directory

**What this means:**
- **Kernel failures block merge**: Any failing must_have_ac AC = CI failure
- **Kernel unknowns block main**: Any unknown must_have_ac AC = selftest failure on main
- **Optional unknowns are informational**: Non-kernel ACs may be unknown without blocking
- **Coverage baseline is advisory**: 80% is a floor, not a ceiling

## 4. Commands Reference

| Command | Purpose |
|---------|---------|
| `cargo xtask ac-status` | Show AC coverage status (summary or JSON) |
| `cargo xtask ac-coverage` | Show coverage backlog (--todo, --must-have) |
| `cargo xtask ac-report` | Generate human-readable coverage report |
| `cargo xtask ac-history` | Manage coverage snapshots |
| `cargo xtask ac-slo` | Check SLOs against snapshots |
| `cargo xtask ac-suggest-scenarios` | Generate BDD scenario scaffold for AC |
| `cargo xtask ac-new` | Create new AC in spec_ledger.yaml |
| `cargo xtask ac-tests` | List all tests mapped to an AC |
| `cargo xtask test-ac` | Run tests for a specific AC |

## 5. For Fork Maintainers

### 5.1 Adding New ACs

1. Add AC to `specs/spec_ledger.yaml` under appropriate REQ
2. Set `must_have_ac: true` for kernel behavior, `false` for optional
3. Run `cargo xtask ac-suggest-scenarios AC-ID` to scaffold scenario
4. Add BDD scenario to `specs/features/*.feature` with `@AC-ID` tag
5. Implement step definitions if needed
6. Verify with `cargo xtask ac-status` and `cargo xtask test-ac AC-ID`

### 5.2 Deciding must_have_ac

Use these criteria:
- **true (kernel)**: Core functionality, security properties, API contracts
- **false (optional)**: Documentation ACs, guidance policies, future plans, ergonomics

### 5.3 CI Integration

Wire these commands into your CI:
```yaml
# Required for all PRs
- cargo xtask selftest

# On main branch
- XTASK_STRICT_AC_COVERAGE=1 cargo xtask selftest
- cargo xtask ac-slo

# Optional: snapshot for trend analysis
- cargo xtask ac-history snapshot
```

## 6. Related Documentation

- `docs/design/ac-coverage-format.md` - JSONL format specification
- `docs/reference/xtask-commands.md` - Full command reference
- `ADR-0023` - Coverage JSONL as primary source
- `ADR-0005` - Selftest as single gate

## 7. Triage Status

As of this document, the AC backlog is fully classified:

- **62 kernel ACs**: All passing (100% kernel coverage)
- **55 optional ACs**: 37 passing, 18 unknown (intentionally not gating)

The 18 optional unknown ACs fall into these categories:
- **Governance documentation** (validated by CI, not BDD)
- **Guidance policies** (lint warnings, not hard enforcement)
- **CI gates** (validated by CI workflows, not BDD scenarios)

All optional unknown ACs have `note:` fields in `spec_ledger.yaml` explaining their classification.
