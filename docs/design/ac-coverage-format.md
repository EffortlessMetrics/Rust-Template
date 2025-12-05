# AC Coverage JSONL Format

This document specifies the line-oriented JSON format written to:

- `target/ac/coverage.jsonl`

The file is produced by the `AcCoverageWriter` during BDD acceptance test runs
and is the **primary truth source** for AC coverage used by:

- `cargo xtask ac-status`
- `cargo xtask selftest` (step 10: AC coverage gate)

## File Location and Lifecycle

- Path: `target/ac/coverage.jsonl`
- Lifecycle:
  - Created/overwritten on each acceptance test run.
  - Not meant to be committed to version control.
- Multiple runs append/overwrite according to the acceptance harness; consumers
  treat the file as a snapshot for the latest BDD run.

## Record Schema

The file is **JSON Lines** (one JSON object per line).

Each line is an `AcCoverageRecord`:

```json
{
  "ac_id": "AC-KERN-001",
  "status": "passed",
  "feature": "specs/features/governance/health.feature",
  "scenario": "Dashboard shows governance health",
  "tags": ["AC-KERN-001", "smoke", "tier1"]
}
```

Fields:

- `ac_id` (string)
  - Acceptance Criteria identifier.
  - Typically matches the IDs in `specs/spec_ledger.yaml` (e.g. `AC-KERN-001`,
    `AC-PLT-HEALTH-001`).
  - Extracted from Gherkin tags that start with `AC-` or `ac-`.

- `status` (string)
  - Scenario result status:
    - `"passed"`
    - `"failed"`
    - `"skipped"`
  - This is **scenario-level**, not AC-level.

- `feature` (string)
  - Path to the feature file as seen by the acceptance runner
    (e.g. `specs/features/git_hooks.feature`).

- `scenario` (string)
  - Scenario name from the feature file.

- `tags` (array of string)
  - All tags applied to the scenario, including tags inherited from `Feature`
    and `Rule` levels.
  - Includes AC IDs and additional tags such as `smoke`, `tier1`, etc.

## Scenario Identity

When aggregating coverage, scenarios are identified by a composite key:

```text
feature :: scenario
```

Example keys:

- `specs/features/git_hooks.feature::Pre-commit hook is executable`
- `specs/features/devex.feature::Dev-up bootstraps the environment`

This avoids collisions when the same scenario name appears in multiple feature
files.

## Aggregation Semantics

The aggregation logic used by `ac-status`:

- Per scenario:
  - `"passed"` -> contributes a `true` for that `ac_id`
  - `"failed"` -> contributes a `false` for that `ac_id`
  - `"skipped"` -> **ignored** for aggregation (treated as "not proven")

- Per AC:
  - At least one `failed` scenario -> **Fail**
  - One or more `passed` scenarios, and no failures -> **Pass**
  - No entries (only skipped, or no scenarios at all) -> **Unknown**

Unknown is surfaced when merging coverage with the spec ledger:

- AC present in ledger but not in `coverage.jsonl` -> Unknown
- AC present with only skipped scenarios -> Unknown

## Backwards Compatibility

Changes to this format should be treated as a contract change:

- Prefer **additive** changes (new fields) over breaking existing ones.
- If you change field semantics, bump an explicit `format_version` in this doc
  and in any consumers that rely on the current behavior.

---

**Format Version:** 1.0
**Last Updated:** 2025-12-05
