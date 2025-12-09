# ADR-0023: AC Coverage JSONL as Primary Truth Source

<!-- doclint:disable orphan-version -->
<!-- Historical: References cucumber-rs version 0.21.1 as documentation context -->

**Status**: Accepted
**Date**: 2025-12-05
**Authors**: Steven Zimmerman
**Related ACs**: AC-PLT-COVERAGE-001

---

## Context

The acceptance test harness uses cucumber-rs 0.21.1 which has a known issue: it calls `std::process::exit()` at the end of test runs, which prevents proper buffer flushing for writers that rely on Drop semantics.

Previously, AC coverage was derived from:
1. JUnit XML (`target/junit/acceptance.xml`) - written by cucumber's JUnit writer
2. Cucumber JSON report (`target/ac_report.json`) - written by cucumber's JSON writer

Both of these outputs rely on Drop being called to flush their buffers. When cucumber calls `exit()`, these buffers are not flushed, resulting in empty or incomplete files.

This caused several issues:
- `ac-status` reported stale or incorrect AC statuses
- `docs/feature_status.md` showed outdated coverage information
- CI gates could not reliably determine which ACs were passing

---

## Decision

Introduce `AcCoverageWriter` in the acceptance harness that writes coverage results to `target/ac/coverage.jsonl` in a streaming fashion. This writer:

1. Flushes to disk on each scenario completion (not in Drop)
2. Uses JSON Lines format (one JSON object per line)
3. Includes all metadata: AC ID, status, feature file, scenario name, and tags

The coverage.jsonl file is now the **primary truth source** for AC coverage. The tooling cascade is:

1. **Primary**: `target/ac/coverage.jsonl` (resilient to exit())
2. **Fallback 1**: `target/ac_report.json` (Cucumber JSON)
3. **Fallback 2**: `target/junit/acceptance.xml` + feature file parsing (legacy)

Both `ac-status` and `ac-coverage` commands now consume `parse_ac_coverage()` from the shared `ac_parsing` module.

### Alternatives Considered

1. **Patch cucumber-rs**: Would require forking the crate and maintaining a patch. Rejected due to maintenance burden.

2. **Signal handler to flush buffers**: Complex, platform-specific, and may not work reliably with exit().

3. **Post-process empty files**: Would require re-running tests, not a true fix.

---

## Consequences

### Positive

- AC coverage is reliably captured even when cucumber exits abnormally
- Single source of truth for coverage data
- Format is documented in `docs/design/ac-coverage-format.md`
- All coverage consumers use the same parsing logic
- Easy to debug: JSONL is human-readable and line-oriented

### Negative

- Additional file to manage (`target/ac/coverage.jsonl`)
- Slight overhead from per-scenario flush (negligible in practice)
- JUnit/JSON fallback paths add code complexity

### Neutral

- JUnit XML is still generated for CI systems that expect it
- Existing CI workflows continue to work with the fallback path

---

## Compliance

Automated enforcement:

1. `cargo xtask selftest` step 10 validates AC coverage via `ac-status --summary`
2. `ac-status` and `ac-coverage` both use `parse_ac_coverage()` as primary
3. The coverage.jsonl format is documented and validated by `parse_ac_coverage()`
4. Selftest enforces `must_have_ac=true` coverage:
   - Default: Failing ACs fail the gate, Unknown ACs are advisory
   - Strict mode (`XTASK_STRICT_AC_COVERAGE=1`): Unknown `must_have_ac` ACs also fail the gate

View the coverage backlog with:
```bash
cargo xtask ac-coverage --todo              # All unknown ACs
cargo xtask ac-coverage --todo --must-have  # Only kernel (must_have_ac) ACs
```

Manual review:

- Changes to `AcCoverageWriter` or `parse_ac_coverage()` should be reviewed for format compatibility
- Breaking changes to the JSONL format require version bump in `docs/design/ac-coverage-format.md`

---

## Notes

### Related Files

- `crates/acceptance/src/coverage_writer.rs` - AcCoverageWriter implementation
- `crates/xtask/src/commands/ac_parsing.rs` - parse_ac_coverage() function
- `docs/design/ac-coverage-format.md` - Format specification (version 1.0)

### Known Issue

JUnit XML from cucumber-rs 0.21.1 may still be empty due to the exit() behavior. This is a known upstream issue. The coverage.jsonl workaround makes JUnit a best-effort artifact rather than the source of truth.

### Migration

No migration required. The new primary path is automatically used when coverage.jsonl exists. Existing tooling falls back gracefully when it doesn't.
