---
id: GUIDE-TPL-SELECTIVE-TESTING-001
title: Selective Testing Guide
doc_type: guide
status: draft
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT]
acs: []
adrs: [ADR-0005]
---

# Selective Testing Guide

**Purpose:** Run only the tests affected by your changes, avoiding slow full test suite runs on Tier-2 platforms (native Windows).

## Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `cargo xtask test-ac <AC-ID>` | Test single acceptance criterion | `cargo xtask test-ac AC-PLT-001` |
| `cargo xtask test-changed [--base <ref>]` | Test only changed files | `cargo xtask test-changed` |
| `cargo xtask check` | Fast core checks (fmt, clippy, unit tests, change-aware BDD) | `cargo xtask check` |
| `cargo xtask selftest` | Full validation (Tier-1 only) | `nix develop && cargo xtask selftest` |

**Default validation ladder**
- After edits: `cargo xtask test-changed`
- Specific AC touched: `cargo xtask test-ac <AC-ID>`
- Pre-merge (Tier-1): `nix develop && cargo xtask selftest`

---

## The Problem

Running the full BDD suite (`cargo xtask bdd`) or selftest on **native Windows** can take 2+ hours due to:
- Executable file locking (`xtask.exe`, `acceptance.exe`)
- Antivirus scanning `target/` directory
- Serial execution of 100+ acceptance scenarios

On **Tier-1 platforms** (Nix + Linux/macOS/WSL2), this is much faster (10-20 minutes), but still too slow for rapid iteration.

---

## The Solution: Selective Testing

### 1. Test a Single Acceptance Criterion

When working on a specific AC:

```bash
# Example: Testing the doctor command
cargo xtask test-ac AC-PLT-001

# What it does:
# 1. Looks up AC-PLT-001 in specs/spec_ledger.yaml
# 2. Finds all BDD scenarios tagged with @AC-PLT-001
# 3. Runs only those scenarios
# 4. Reports pass/fail
```

**Output example:**
```
[INFO] Looking up AC: AC-PLT-001
[INFO] Found AC: AC-PLT-001 (requirement: REQ-PLT-ONBOARDING)

[INFO] Searching for BDD scenarios tagged with @AC-PLT-001...
[INFO] Found 1 scenario(s):
  - doctor validates the environment and prints next steps

[INFO] Running acceptance tests for @AC-PLT-001...

[PASS] All tests passed for AC-PLT-001
       Scenarios: 1
```

### 2. Test Only What Changed

After making file edits:

```bash
# Compare against origin/main (default)
cargo xtask test-changed

# Compare against different base
cargo xtask test-changed --base HEAD~5
cargo xtask test-changed --base main
```

**Smart file classification:**
- `specs/features/*.feature` -> Runs BDD for ACs in that feature
- `specs/spec_ledger.yaml` -> Runs all BDD tests
- `crates/xtask/**` -> Runs xtask unit tests + DevEx BDD
- `crates/app-http/**` -> Runs app-http unit tests
- `crates/spec-runtime/**` -> Runs spec-runtime tests + graph invariants
- `crates/business-core/**` -> Runs business-core tests
- `docs/**` only -> Runs `docs-check` (lightweight)

**Output example:**
```
Analyzing changed files...

Changed files (vs origin/main):
  - specs/features/xtask_devex.feature
  - crates/xtask/src/commands/dev_up.rs

Test plan:
  1. Run BDD: @AC-PLT-018 (Changed feature files)
  2. Run unit tests: xtask (xtask crate changes)

Executing tests:
  [run] cargo test -p acceptance --test acceptance (tags: "@AC-PLT-018")
    [ok] Changed feature files
  [run] cargo test -p xtask
    [ok] xtask crate changes

[ok] All tests passed
```

**Tag syntax**
- Feature files should tag scenarios as `@AC-...`
- `CUCUMBER_TAG_EXPRESSION` accepts `AC-...` or `@AC-...`; `test-changed` normalizes to `@AC-...` when invoking Cucumber

**Plan-only preview**
- `XTASK_TEST_CHANGED_PLAN_ONLY=1 XTASK_CHANGED_BASE=HEAD cargo xtask test-changed` prints the change-aware plan and tag expression without executing tests (handy for acceptance tests or slow machines)

**No changes detected:**
```
Analyzing changed files...

Changed files (vs origin/main):
  - README.md

No tests needed for these changes.
```

---

## Recommended Workflow by Platform

### Tier-1 (Nix + Linux/macOS/WSL2)

**Fast iteration:**
```bash
# Make changes
cargo xtask test-changed

# Or test specific AC
cargo xtask test-ac AC-PLT-001

# Before commit
cargo xtask selftest
```

**Pre-commit hook:**
```bash
#!/usr/bin/env bash
set -e
cargo xtask test-changed
```

### Tier-2 (Native Windows)

**Fast iteration:**
```bash
# Make changes
cargo xtask check  # fmt, clippy, unit tests only

# Test specific AC (if applicable)
cargo xtask test-ac AC-PLT-001

# Or test changed files
cargo xtask test-changed
```

**Pre-commit hook:**
```bash
#!/usr/bin/env bash
set -e
cargo xtask test-changed
```

**Full validation (use Tier-1):**
```bash
# Switch to WSL2 or Linux VM
nix develop
cargo xtask selftest
```

---

## When to Run Full Selftest

Reserve `cargo xtask selftest` for:

1. **CI/CD pipelines** (automated, Tier-1 environment)
2. **Pre-merge validation** (in Tier-1 environment)
3. **Release candidates** (comprehensive gate)
4. **After major refactoring** (rare, manual trigger)

**Never:**
- [FAIL] On every commit (use selective testing)
- [FAIL] On native Windows for iteration (use Tier-1 for full validation)
- [FAIL] For single-file changes (use `test-ac` or `test-changed`)

---

## Performance Comparison

| Environment | Command | Time | Use Case |
|-------------|---------|------|----------|
| Native Windows | `cargo xtask selftest` | 2+ hours | [FAIL] Avoid |
| Native Windows | `cargo xtask test-changed` | 30s - 5m | [OK] Local iteration |
| Native Windows | `cargo xtask test-ac AC-PLT-001` | 10s - 2m | [OK] Focused work |
| WSL2 + Nix | `cargo xtask selftest` | 10-20m | [OK] Pre-merge gate |
| WSL2 + Nix | `cargo xtask test-changed` | 30s - 3m | [OK] Fast loop |
| CI (Linux + Nix) | `cargo xtask selftest` | 10-20m | [OK] Canonical truth |

---

## Error Handling

### "No AC found"
```bash
$ cargo xtask test-ac AC-MISSING-001
Error: AC not found in specs/spec_ledger.yaml: AC-MISSING-001

Suggestions (first 10 ACs):
  - AC-PLT-001
  - AC-PLT-002
  ...
```

**Fix:** Check `specs/spec_ledger.yaml` for correct AC-ID, or create it:
```bash
cargo xtask ac-new AC-MYSERV-001 "Description" --requirement REQ-ID
```

### "No scenarios found for AC"
```bash
$ cargo xtask test-ac AC-PLT-999
[WARN] No BDD scenarios found for AC-PLT-999
[INFO] Use: cargo xtask ac-suggest-scenarios AC-PLT-999
```

**Fix:** Add scenarios to feature file or use:
```bash
cargo xtask ac-suggest-scenarios AC-PLT-999
```

### "Git repository not found"
```bash
$ cargo xtask test-changed
Error: Not a git repository
```

**Fix:** Ensure you're in the repository root with `.git/` directory.

### "Cannot determine base ref"
```bash
$ cargo xtask test-changed --base nonexistent-branch
Error: Git ref not found: nonexistent-branch
```

**Fix:** Use valid git ref:
```bash
git branch -a  # List all branches
cargo xtask test-changed --base origin/main
```

---

## Integration with Pre-Commit Hooks

The git pre-commit hook can use selective testing:

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit

set -e

echo "Running selective tests on changed files..."
cargo xtask test-changed

echo "[OK] Pre-commit checks passed"
```

Install/update hook:
```bash
cargo xtask install-hooks
```

---

## Advanced Usage

### Testing Multiple ACs

```bash
# Sequential (current approach)
cargo xtask test-ac AC-PLT-001
cargo xtask test-ac AC-PLT-002

# Or use BDD tag syntax
cargo test -p acceptance --test acceptance -- --tags "@AC-PLT-001 or @AC-PLT-002"
```

### Plan-only (no test execution)

```bash
XTASK_TEST_CHANGED_PLAN_ONLY=1 cargo xtask test-changed
```

Shows the change-aware plan and `CUCUMBER_TAG_EXPRESSION` without running any tests. Combine with `XTASK_CHANGED_BASE=HEAD` for fully local diffs.

### Custom Test Plans

For complex changes spanning multiple crates:

```bash
# 1. Check what would run
cargo xtask test-changed --dry-run  # (future enhancement)

# 2. Manually compose
cargo test -p xtask
cargo test -p spec-runtime
cargo test -p acceptance --test acceptance -- --tags "@AC-PLT-001"
```

### CI Integration

```yaml
# .github/workflows/ci.yml (example)
name: CI

on: [push, pull_request]

jobs:
  selective-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Needed for git diff

      - name: Run selective tests
        run: |
          nix develop --command cargo xtask test-changed --base origin/main

  full-validation:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3

      - name: Full selftest
        run: |
          nix develop --command cargo xtask selftest
```

---

## Troubleshooting

### File Lock Errors on Windows

```
error: failed to remove file 'target\debug\xtask.exe'
Caused by: Access is denied. (os error 5)
```

**Causes:**
- Old cargo/xtask process still running
- VS Code rust-analyzer holding locks
- Multiple terminals running builds

**Fixes:**
```powershell
# Kill all cargo processes
taskkill /F /IM cargo.exe /IM xtask.exe /T

# Restart terminal
# Close VS Code and reopen
```

### Test Passed Locally But Failed in CI

**Likely cause:** Using Tier-2 (Windows) for local testing, CI uses Tier-1 (Linux).

**Fix:** Validate in matching environment:
```bash
# In WSL2 with Nix (matches CI)
nix develop
cargo xtask test-changed
cargo xtask selftest
```

### Slow Test Execution

If `test-changed` is slow (>5 minutes):

1. **Check what's running:**
   ```bash
   # Add --verbose flag (future enhancement)
   cargo xtask test-changed --verbose
   ```

2. **Verify impact analysis:**
   - Changed `specs/spec_ledger.yaml`? Triggers all BDD tests
   - Changed core crate? Triggers many dependent tests

3. **Use more targeted command:**
   ```bash
   # Instead of test-changed for ledger changes
   cargo xtask test-ac AC-SPECIFIC-ID
   ```

---

## Implementation Details

### How `test-ac` Works

1. Parses `specs/spec_ledger.yaml` to find AC definition
2. Uses `ac_parsing` module to extract metadata (requirement, tests)
3. Scans `specs/features/*.feature` for scenarios tagged with `@<AC-ID>`
4. Executes: `cargo test -p acceptance --test acceptance -- --tags @<AC-ID>`
5. Reports results with scenario count

### How `test-changed` Works

1. Executes: `git diff --name-only <base>...HEAD`
2. Classifies changed files by prefix/pattern
3. Maps to test actions:
   - Feature files -> Extract AC tags -> Run BDD
   - Crate sources -> Run unit tests for that crate
   - Docs -> Run lightweight docs-check
4. Deduplicates test commands
5. Executes plan sequentially
6. Aggregates pass/fail results

### File Classification Rules

```rust
// Pseudocode
match file_path {
    "specs/spec_ledger.yaml" => run_all_bdd(),
    "specs/features/*.feature" => run_bdd_for_acs_in_file(),
    "crates/xtask/**" => run_xtask_tests() + run_devex_bdd(),
    "crates/app-http/**" => run_app_http_tests(),
    "crates/spec-runtime/**" => run_spec_runtime_tests() + run_graph_invariants(),
    "crates/business-core/**" => run_business_core_tests(),
    "crates/acceptance/**" => run_all_bdd(),
    "docs/**" => run_docs_check(),
    _ => skip(),
}
```

---

## Future Enhancements

Potential additions (not yet implemented):

1. **Parallel execution:** Run independent test groups concurrently
2. **First-class plan-only flag:** Promote `XTASK_TEST_CHANGED_PLAN_ONLY` to `--plan-only`
3. **Test caching:** Skip tests for unchanged code
4. **Impact graph:** Use cargo metadata to detect transitive impacts
5. **Watch mode:** `--watch` to re-run on file changes
6. **JSON output:** `--json` for tooling integration
7. **Test result caching:** Skip re-running passing tests

---

## Related Commands

| Command | Documentation |
|---------|---------------|
| `cargo xtask check` | Core checks (fmt, clippy, tests) |
| `cargo xtask bdd` | Full BDD suite |
| `cargo xtask selftest` | Complete validation (7 steps) |
| `cargo xtask ac-coverage` | Show which ACs have tests |
| `cargo xtask ac-status` | Detailed AC test status |
| `cargo xtask ac-suggest-scenarios` | Generate BDD scenarios for AC |

---

## See Also

- [AGENT_GUIDE.md](./AGENT_GUIDE.md) - Full agent workflow
- [MISSING_MANUAL.md](./MISSING_MANUAL.md) - Platform support tiers
- [Platform support docs](./explanation/platform-support.md)
- [BDD workflow](./how-to/write-bdd-tests.md)
