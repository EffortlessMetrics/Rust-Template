# Tutorial: Ship Your First AC-driven Change

This tutorial shows how to add or update an acceptance criterion (AC), wire
it to a scenario, and implement code and tests to satisfy it.

Steps (sketch):
1. Pick or add an AC in `specs/spec_ledger.yaml`.
2. Add or update a scenario in `specs/features/*.feature` with `@AC-####`.
3. Implement or adjust Rust code in `crates/core` (and friends).
4. Run acceptance tests locally.
5. Open a PR and verify `ACs`, `Gherkin`, and related checks.

