# Quick Start: Coverage and Build Metrics

**Quick reference for the new coverage and build time tracking features.**

## Test Coverage

### Run Coverage Analysis

```bash
cargo xtask coverage
```

**Output:**
```
Running test coverage analysis

Generating coverage report (this may take a while)...

Coverage Report:
  Coverage: 68.00%
  Baseline: 65.00%

✓ Coverage target met! (68.00% >= 65.00%)
```

### Generate HTML Report

```bash
cargo tarpaulin --out Html --exclude-files tests --workspace --exclude acceptance --exclude xtask
open target/coverage/index.html
```

### Coverage in Selftest

Coverage is automatically checked in `cargo xtask selftest` as step 11/11:

```bash
cargo xtask selftest

# Step 11/11: Test coverage (advisory)
# ⚠ Test coverage below baseline (advisory)
# 💡 Hint: Run `cargo xtask coverage` for detailed coverage report
```

**Note:** Coverage check is a **soft gate** - it won't fail selftest, only warn.

---

## Build Time Tracking

### Capture Build Metrics

```bash
# Clean build to get accurate metrics
cargo xtask build-time-capture

# Output saved to: build_times.json
```

**Metrics Captured:**
- Total build time (seconds)
- Git SHA
- Template version
- Binary sizes (debug/release, if available)
- Codegen/linker time (future enhancement)

### Compare Two Builds

```bash
# Save baseline
cargo xtask build-time-capture
mv build_times.json baseline.json

# Make changes, capture new metrics
cargo xtask build-time-capture

# Compare
cargo xtask build-time-compare --baseline baseline.json --current build_times.json
```

**Output:**
```
Comparing build time metrics

Baseline:
  Version: 3.3.5 (abc123...)
  Total time: 150.00s

Current:
  Version: 3.3.6 (def456...)
  Total time: 156.42s

Comparison:
  Total time: slower (+6.42s, +4.3%)
  Release size: smaller (-2.30 MB, -1.5%)
```

### CI Usage (Future)

```yaml
# Save baseline on main branch
- name: Capture baseline
  run: cargo xtask build-time-capture
- uses: actions/upload-artifact@v4
  with:
    name: baseline-build-times
    path: build_times.json

# Compare on PR
- uses: actions/download-artifact@v4
  with:
    name: baseline-build-times
- name: Compare build times
  run: |
    cargo xtask build-time-capture
    cargo xtask build-time-compare --baseline baseline.json --current build_times.json
```

---

## Feature Flags

### Available Features

See `/docs/FEATURE_FLAG_TEST_MATRIX.md` for complete documentation.

**Quick reference:**

| Crate | Feature | Usage |
|-------|---------|-------|
| `adapters-grpc` | `integration-grpc` | `cargo test -p adapters-grpc --features integration-grpc` |
| `adapters-db-sqlx` | `integration-db` | `cargo test -p adapters-db-sqlx --features integration-db` |
| `telemetry` | `otlp` | `cargo build -p telemetry --features otlp` |

---

## Commands at a Glance

```bash
# Coverage
cargo xtask coverage              # Run coverage analysis (baseline: 65%)

# Build Time
cargo xtask build-time-capture    # Capture build metrics
cargo xtask build-time-compare \
  --baseline old.json \
  --current new.json              # Compare two builds

# Selftest (includes coverage)
cargo xtask selftest              # Step 11/11: test coverage (soft gate)
```

---

## Files

**New Files:**
- `/crates/xtask/src/commands/coverage.rs` - Coverage command implementation
- `/crates/xtask/src/commands/build_time.rs` - Build time tracking implementation
- `/docs/FEATURE_FLAG_TEST_MATRIX.md` - Feature flag documentation
- `/IMPLEMENTATION_SUMMARY.md` - Complete implementation details

**Modified Files:**
- `/crates/xtask/src/main.rs` - Added CLI commands
- `/crates/xtask/src/commands/mod.rs` - Added module exports
- `/crates/xtask/src/commands/selftest.rs` - Added coverage step (11/11)
- `/specs/devex_flows.yaml` - Added command metadata

---

## Related Documentation

- `IMPLEMENTATION_SUMMARY.md` - Full implementation details
- `docs/FEATURE_FLAG_TEST_MATRIX.md` - Feature flag test coverage
- `docs/CLAUDE.md` - Template workflows and governance
- `.github/workflows/tier1-selftest.yml` - CI selftest workflow

---

**Status:** ✅ Ready to Use
