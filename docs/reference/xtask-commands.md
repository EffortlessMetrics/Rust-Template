# Reference: xtask Commands

Complete reference for all `xtask` CLI commands.

**Quick Index:**
- [check](#xtask-check) - Format, lint, test
- [bdd](#xtask-bdd) - Run BDD acceptance tests
- [bundle](#xtask-bundle) - Generate LLM context
- [quickstart](#xtask-quickstart) - First-run validation
- [selftest](#xtask-selftest) - Comprehensive validation suite

---

## xtask check

Run all code quality checks: formatting, linting, and tests.

### Usage

```bash
cargo run -p xtask -- check

# Or in Nix shell
nix develop -c cargo run -p xtask -- check
```

### What It Does

Runs three checks in sequence:

1. **Format check:** `cargo fmt --all -- --check`
2. **Lint check:** `cargo clippy --all-targets --all-features -- -D warnings`
3. **Unit tests:** `cargo test --workspace --exclude acceptance`

Stops at first failure.

### Exit Codes

- `0`: All checks passed
- Non-zero: One or more checks failed

### When to Use

- **Before every commit** - Ensure code quality
- **In pre-commit hooks** - Automatic validation
- **In CI** - Required check for pull requests

### Example Output

```
Running format check...
Running clippy...
Running tests...
   Running unittests src/lib.rs (target/debug/deps/core-...)
   Running unittests src/lib.rs (target/debug/deps/model-...)
✓ All checks passed
```

### Common Issues

**Format check fails:**
```bash
# Fix by running
cargo fmt --all
```

**Clippy warnings:**
```bash
# See specific warnings
cargo clippy --all-targets --all-features
```

**Tests fail:**
```bash
# Run tests with output
cargo test --workspace --exclude acceptance -- --nocapture
```

---

## xtask bdd

Run BDD acceptance tests with JUnit XML output.

### Usage

```bash
cargo run -p xtask -- bdd
```

### What It Does

1. Runs `cargo test -p acceptance --test acceptance`
2. Generates JUnit XML at `target/junit/acceptance.xml`
3. Prints scenario results to console

### Exit Codes

- `0`: All scenarios passed
- Non-zero: One or more scenarios failed

### Output Artifacts

- `target/junit/acceptance.xml` - JUnit test results for CI

### When to Use

- **After implementing AC** - Verify scenario passes
- **Before pull request** - Ensure acceptance criteria met
- **In CI** - Validate behavioral requirements

### Example Output

```
Running acceptance tests...
Feature: Refunds
  Scenario: Create a refund
   ✔  Given an order "ORD-1" totalling 5000 cents
   ✔  When I POST /refunds with { "orderId": "ORD-1", "amountCents": 5000 }
   ✔  Then I receive 201 with a "refundId"
[Summary]
1 feature
1 scenario (1 passed)
3 steps (3 passed)
✓ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

### Common Issues

**Scenario fails:**
- Check step definitions in `crates/acceptance/src/steps/`
- Verify feature file syntax in `specs/features/`
- Run with verbose output: `cargo test -p acceptance -- --nocapture`

**JUnit XML not generated:**
- Check `target/junit/` directory exists
- Verify path in `crates/acceptance/tests/acceptance.rs`

---

## xtask bundle

Generate LLM context bundle for a specific task.

### Usage

```bash
cargo run -p xtask -- bundle <task>

# Examples
cargo run -p xtask -- bundle implement_ac
cargo run -p xtask -- bundle implement_feature
cargo run -p xtask -- bundle debug_tests
```

### What It Does

1. Reads `.llm/contextpack.yaml` for task configuration
2. Resolves `include` glob patterns via `git ls-files`
3. Respects `.llm/.llmignore` exclusions
4. Enforces `max_bytes` limit
5. Generates markdown bundle at `.llm/bundle/<task>.md`

### Parameters

- `<task>` - Task name defined in `.llm/contextpack.yaml`

### Exit Codes

- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

### Output Artifacts

- `.llm/bundle/<task>.md` - Generated context bundle

### When to Use

- **Before LLM coding session** - Get focused context
- **When implementing AC** - Provide specs + tests + code
- **When debugging** - Include relevant test failures

### Example Output

```
Generating LLM context bundle for task: implement_ac
Building context bundle: implement_ac
  Max size: 250000 bytes
  Description: Context for implementing an AC: ledger, specs, features, and core code
  Files included: 6
  Bundle size: 2708 bytes

Bundle written to: /path/to/.llm/bundle/implement_ac.md
✓ Bundle generated: .llm/bundle/implement_ac.md
```

### Common Issues

**Task not found:**
- Check `.llm/contextpack.yaml` has task defined
- Verify task name spelling

**Bundle too large:**
- Reduce `max_bytes` in contextpack.yaml
- Make `include` patterns more specific

**Missing files:**
- Files must be tracked by git (`git ls-files`)
- Check `.llm/.llmignore` isn't excluding needed files

### Configuration

Edit `.llm/contextpack.yaml`:

```yaml
tasks:
  my_task:
    max_bytes: 150000
    include:
      - specs/spec_ledger.yaml
      - crates/core/src/**/*.rs
    description: "Custom task description"
```

---

## xtask quickstart

Quick validation of template functionality (first-run check).

### Usage

```bash
cargo run -p xtask -- quickstart
```

### What It Does

1. Checks environment (cargo, rustc versions)
2. Runs `xtask check`
3. Runs `xtask bdd`
4. Runs `xtask bundle implement_ac`
5. Reports results with colored output

### Exit Codes

- `0`: All validation passed
- Non-zero: One or more validation steps failed

### When to Use

- **First time** - After cloning template
- **After setup changes** - Verify environment still works
- **Quick health check** - Lighter than `selftest`

### Example Output

```
======================================
  Rust Template Quick Start
======================================

[1/5] Checking environment...
  ✓ cargo 1.91.0
  ✓ rustc 1.91.0

[2/5] Running xtask check...
  ✓ Format check passed
  ✓ Clippy passed
  ✓ Tests passed

[3/5] Running BDD acceptance tests...
  ✓ BDD scenarios passed
  ✓ JUnit output created

[4/5] Testing LLM context bundler...
  ✓ Bundle command executed
  ✓ Bundle created (2708 bytes)

[5/5] Testing helper commands...
  ✓ Core commands validated

======================================
✓ Template validation passed!

Next steps:
  • See docs/how-to/new-service-from-template.md for adoption guide
  • See TEMPLATE_API.md for stable interface documentation
  • See docs/tutorials/first-ac-change.md for AC-first development
======================================
```

### Difference from `selftest`

| Feature | quickstart | selftest |
|---------|-----------|----------|
| Environment check | ✓ | ✓ |
| Core checks | ✓ | ✓ |
| BDD tests | ✓ | ✓ |
| AC status | ✗ | ✓ |
| Policy tests | ✗ | ✓ (if available) |
| **Use case** | First run | CI/comprehensive |

---

## xtask selftest

Complete template self-test suite (used in CI).

### Usage

```bash
cargo run -p xtask -- selftest

# Or in Nix shell
nix develop -c cargo run -p xtask -- selftest
```

### What It Does

Comprehensive validation in 5 steps:

1. **Core checks:** format, clippy, tests
2. **BDD tests:** acceptance scenarios + JUnit XML
3. **AC status:** Maps tests → ACs, generates feature_status.md
4. **LLM bundler:** Validates context generation
5. **Policy tests:** Runs Rego policies (if conftest available)

### Exit Codes

- `0`: All self-tests passed
- Non-zero: One or more test suites failed

### Output Artifacts

- `target/junit/acceptance.xml` - JUnit test results
- `docs/feature_status.md` - AC status mapping
- `.llm/bundle/implement_ac.md` - LLM context bundle

### When to Use

- **In CI/CD** - Comprehensive validation
- **Before releases** - Full health check
- **After major changes** - Verify nothing broke

### Example Output

```
======================================
  Template Self-Test Suite
======================================

[1/5] Running core checks (fmt, clippy, tests)...
  ✓ Core checks passed

[2/5] Running BDD acceptance tests...
  ✓ BDD scenarios passed
  ✓ JUnit XML generated

[3/5] Running AC status mapping...
  ✓ AC status script executed
  ✓ Feature status generated

[4/5] Testing LLM context bundler...
  ✓ Bundle generated
  ✓ Bundle size: 2708 bytes

[5/5] Running policy tests...
  ⚠ Policy tests: conftest not available
  ℹ (Run 'nix develop' for full policy validation)

======================================
✓ All self-tests passed!

The template is working correctly:
  • xtask commands functional
  • BDD scenarios passing
  • AC mapping operational
  • LLM bundler working

Ready for:
  • Service development: docs/how-to/new-service-from-template.md
  • AC-first workflow: docs/tutorials/first-ac-change.md
======================================
```

### CI Integration

Used in `.github/workflows/ci-template-selftest.yml`:

```yaml
- name: Run template self-test suite
  run: nix develop -c cargo run -p xtask -- selftest
```

### Graceful Degradation

- AC status failures are warnings (informational)
- Policy tests skip if conftest unavailable
- Only core checks and BDD are hard requirements

---

## Command Comparison

| Command | Speed | Coverage | Use Case |
|---------|-------|----------|----------|
| `check` | Fast | Code quality | Every commit |
| `bdd` | Medium | Acceptance | After AC work |
| `bundle` | Fast | Context gen | Before LLM use |
| `quickstart` | Medium | Basic validation | First run |
| `selftest` | Slow | Comprehensive | CI, releases |

---

## Environment Variables

### RUST_LOG

Controls tracing output for all commands:

```bash
# Default (INFO level)
cargo run -p xtask -- check

# Verbose (DEBUG level)
RUST_LOG=debug cargo run -p xtask -- check

# Specific crate
RUST_LOG=xtask=trace cargo run -p xtask -- check
```

### CARGO_TERM_COLOR

Controls color output:

```bash
# Force colors (in CI)
CARGO_TERM_COLOR=always cargo run -p xtask -- check

# Disable colors
CARGO_TERM_COLOR=never cargo run -p xtask -- check
```

---

## Tips & Tricks

### Run Multiple Commands

```bash
# Sequential (stops on first failure)
cargo run -p xtask -- check && cargo run -p xtask -- bdd

# Always run both (for CI)
cargo run -p xtask -- check; cargo run -p xtask -- bdd
```

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Re-run on file changes
cargo watch -x 'run -p xtask -- check'
```

### Parallel in CI

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: cargo run -p xtask -- check

  bdd:
    runs-on: ubuntu-latest
    steps:
      - run: cargo run -p xtask -- bdd
```

### Alias for Convenience

Add to `~/.bashrc` or `~/.zshrc`:

```bash
alias xt='cargo run -p xtask --'

# Then use:
xt check
xt bdd
xt bundle implement_ac
```

---

## See Also

- `TEMPLATE_API.md` - Full API specification with schemas
- `docs/how-to/` - Task-oriented guides
- `docs/explanation/architecture.md` - Design rationale
