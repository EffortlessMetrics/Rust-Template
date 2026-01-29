# acceptance – CLAUDE.md

**Tier:** Testing (Layer 9)
**Publish:** No (internal, test-only)
**Dependencies:** cucumber, ac-kernel, tokio

## Purpose

BDD acceptance test suite using Cucumber/Gherkin. Executes feature files and produces AC coverage data.

## Key Exports

None (this is a test crate, not a library)

## Key Files

- `specs/features/*.feature` – Gherkin feature files
- `target/ac/coverage.jsonl` – Coverage output
- `target/junit/acceptance.xml` – JUnit results

## Running

```bash
cargo test -p acceptance
# Or via xtask
cargo xtask test-ac AC-XXX
```

## When to Modify

- Adding new acceptance tests
- Adding step definitions
- Changing test infrastructure

## Architectural Notes

- **Cucumber framework**: Uses cucumber-rs
- **Coverage output**: Writes to coverage.jsonl for ac-kernel
- **AC mapping**: Tests map to ACs via tags

## Consumers

`cargo xtask selftest`, `cargo xtask ac-coverage`

## See Also

- `crates/ac-kernel/` for coverage parsing
- `specs/features/` for feature files
- `specs/spec_ledger.yaml` for AC definitions
