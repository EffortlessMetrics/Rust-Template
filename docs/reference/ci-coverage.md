# Reference: Coverage Workflow

Documents how the `Coverage` workflow works, where the coverage floor is configured, and how to change it safely.

---

## Overview

The coverage workflow generates code coverage reports using `cargo llvm-cov` and enforces a minimum coverage threshold.

**Tooling:**
- `cargo llvm-cov` (via Nix devshell)
- JSON output parsed with `jq`

**Trigger:**
- Runs on release tags only (not on every PR)
- Advisory on PRs (doesn't block merge)

---

## Default Threshold

By default, this template enforces a **60% line coverage floor** for the workspace.

The threshold is configured in `.github/workflows/ci-coverage.yml`:

```yaml
env:
  COVERAGE_THRESHOLD: 60
```

---

## Adjusting the Threshold

### For Brownfield Adoption

Teams adopting the template with existing code may need to start lower:

```yaml
env:
  COVERAGE_THRESHOLD: 40  # Temporary during adoption
```

Document this in your fork's `FRICTION_LOG.md` and plan to ratchet up over time.

### For Mature Services

Production services should aim for 70-80%:

```yaml
env:
  COVERAGE_THRESHOLD: 75
```

---

## Running Coverage Locally

```bash
# Generate coverage report
cargo llvm-cov --workspace --json > coverage.json

# Check threshold manually
jq '.data[0].totals.lines.percent' coverage.json

# Generate HTML report
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```

---

## Coverage Metrics

The workflow reports several metrics:

| Metric | Description |
|--------|-------------|
| **Line coverage** | Percentage of lines executed by tests (primary metric) |
| **Branch coverage** | Percentage of code branches taken |
| **Function coverage** | Percentage of functions called |

Only **line coverage** is enforced by the threshold; others are advisory.

---

## Excluding Code from Coverage

Use `#[cfg(not(coverage))]` or `#[coverage(off)]` for code that shouldn't be measured:

```rust
#[cfg(not(coverage))]
fn debug_only_function() {
    // This won't count against coverage
}
```

Common exclusions:
- Debug/trace logging blocks
- Panic handlers for truly unreachable code
- CLI argument parsing (tested via integration tests)

---

## CI Behavior

| Context | Behavior |
|---------|----------|
| **PR** | Coverage runs but doesn't block merge |
| **Release tag** | Coverage must meet threshold to release |
| **Manual trigger** | Can run on any branch |

---

## Troubleshooting

### "Coverage below threshold"

**Cause**: Not enough test coverage for new code.

**Fix**: Add tests for uncovered lines. Use `cargo llvm-cov --html` to identify gaps.

### Coverage report shows 0%

**Cause**: Tests aren't running or instrumentation failed.

**Fix**: Ensure tests pass first (`cargo test`), then retry coverage.

### "llvm-cov not found"

**Cause**: Not in Nix devshell.

**Fix**: Run `nix develop` first, or install `cargo-llvm-cov` manually.

---

## See Also

- **[ci-workflows.md](ci-workflows.md)** - Full CI workflow reference
- **[testing-strategy.md](../testing-strategy.md)** - Testing philosophy
- **[SELECTIVE_TESTING.md](../SELECTIVE_TESTING.md)** - Selective test execution
