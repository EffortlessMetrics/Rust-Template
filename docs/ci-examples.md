# CI/CD Integration Examples

This document provides complete working examples for integrating the xtask tooling into CI/CD pipelines.

## Table of Contents

- [For Greenfield (Template Users)](#for-greenfield-template-users)
- [For Brownfield (Library Users)](#for-brownfield-library-users)
- [Required Status Checks](#required-status-checks)
- [Estimated CI Run Times](#estimated-ci-run-times)

---

## For Greenfield (Template Users)

If you're using this template directly for a new project, the workflows are already set up for you in `.github/workflows/`.

### Core Workflows Included

#### 1. Self-Test Suite (`.github/workflows/selftest.yml`)

Comprehensive test suite that validates:
- Code formatting (rustfmt)
- Linting (clippy)
- Unit and integration tests
- BDD acceptance tests
- AC status generation
- LLM context bundler
- Policy validation

**Triggers:** Push to main, pull requests

**Usage:** This runs automatically on every PR and push to main.

```yaml
name: Self-Test
on:
  push:
    branches: [main]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  selftest:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    permissions: { contents: read }
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Enable sccache
        run: |
          echo "RUSTC_WRAPPER=$(nix develop -c which sccache)" >> $GITHUB_ENV
          echo "SCCACHE_GHA_ENABLED=1" >> $GITHUB_ENV
          echo "CARGO_INCREMENTAL=0" >> $GITHUB_ENV
          nix develop -c sccache --start-server || true

      - name: Run template self-test suite
        run: nix develop -c cargo run -p xtask -- selftest

      - name: Verify artifacts were generated
        run: |
          test -f docs/feature_status.md || echo "Warning: feature_status.md not generated"
          test -f .llm/bundle/implement_ac.md || echo "Warning: LLM bundle not generated"
          test -f target/junit/acceptance.xml || echo "Warning: JUnit XML not generated"

      - name: Show sccache stats
        if: always()
        run: nix develop -c sccache --show-stats || true

      - name: Upload test artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: selftest-artifacts
          path: |
            docs/feature_status.md
            .llm/bundle/implement_ac.md
            target/junit/acceptance.xml
          if-no-files-found: warn
```

#### 2. Fast Policy Check (`.github/workflows/policy-test.yml`)

Lightweight policy-only validation for quick feedback on policy changes.

**Triggers:** Changes to `policy/**` files

**Usage:** Runs automatically when policy files are modified.

```yaml
name: Policy Tests
on:
  push:
    branches: [main]
    paths:
      - 'policy/**'
      - 'crates/xtask/src/commands/policy_test.rs'
      - '.github/workflows/policy-test.yml'
  pull_request:
    paths:
      - 'policy/**'
      - 'crates/xtask/src/commands/policy_test.rs'
      - '.github/workflows/policy-test.yml'

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  policy-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    permissions: { contents: read }
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Run policy tests
        run: nix develop -c cargo run -p xtask -- policy-test
```

### Additional Specialized Workflows

The template includes many other specialized workflows for comprehensive validation:

- **ci-lints.yml** - Rust linting (clippy, fmt)
- **ci-gherkin.yml** - BDD acceptance tests
- **ci-ac.yml** - Acceptance criteria validation
- **ci-governance.yml** - Governance policy checks
- **ci-security.yml** - Security scanning
- **ci-coverage.yml** - Code coverage reporting
- **ci-msrv.yml** - Minimum supported Rust version check
- **ci-features.yml** - Feature flag validation
- **ci-flags.yml** - Flag policy enforcement

See `.github/workflows/README.md` for details on each workflow.

---

## For Brownfield (Library Users)

If you're integrating `rust_iac_xtask_core` into an existing project, you'll need to create workflows manually.

### Prerequisites

1. Add `rust_iac_xtask_core` to your dependencies (see [integration guide](./how-to/integrate-into-existing-project.md))
2. Create an `xtask` crate in your workspace
3. Set up your `Cargo.toml` workspace

### Minimal Working Example

Create `.github/workflows/xtask-check.yml`:

```yaml
name: XTask Checks
on:
  push:
    branches: [main, develop]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    permissions:
      contents: read
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Run xtask checks
        run: cargo run -p xtask -- check
```

### With Nix Support

If your project uses Nix (recommended for reproducibility):

```yaml
name: XTask Checks (Nix)
on:
  push:
    branches: [main, develop]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    permissions:
      contents: read
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Enable sccache
        run: |
          echo "RUSTC_WRAPPER=$(nix develop -c which sccache)" >> $GITHUB_ENV
          echo "SCCACHE_GHA_ENABLED=1" >> $GITHUB_ENV
          echo "CARGO_INCREMENTAL=0" >> $GITHUB_ENV
          nix develop -c sccache --start-server || true

      - name: Run xtask checks
        run: nix develop -c cargo run -p xtask -- check

      - name: Show sccache stats
        if: always()
        run: nix develop -c sccache --show-stats || true
```

### BDD Acceptance Tests

Create `.github/workflows/xtask-bdd.yml`:

```yaml
name: BDD Acceptance Tests
on:
  push:
    branches: [main, develop]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  bdd:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    permissions:
      contents: read
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Run BDD tests
        run: cargo run -p xtask -- bdd

      - name: Upload JUnit XML
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: junit-results
          path: target/junit/*.xml
          if-no-files-found: warn
```

### Policy Validation

Create `.github/workflows/xtask-policy.yml`:

```yaml
name: Policy Tests
on:
  push:
    branches: [main, develop]
    paths: ['policy/**', '.github/workflows/xtask-policy.yml']
  pull_request:
    paths: ['policy/**', '.github/workflows/xtask-policy.yml']

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  policy:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    permissions:
      contents: read
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install conftest
        run: |
          wget https://github.com/open-policy-agent/conftest/releases/download/v0.49.1/conftest_0.49.1_Linux_x86_64.tar.gz
          tar xzf conftest_0.49.1_Linux_x86_64.tar.gz
          sudo mv conftest /usr/local/bin/
          conftest --version

      - name: Run policy tests
        run: cargo run -p xtask -- policy-test
```

### Full Self-Test Suite

For comprehensive validation (combines all checks):

```yaml
name: Self-Test Suite
on:
  push:
    branches: [main, develop]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  selftest:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    permissions:
      contents: read
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Required for AC status mapping

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Install conftest
        run: |
          wget https://github.com/open-policy-agent/conftest/releases/download/v0.49.1/conftest_0.49.1_Linux_x86_64.tar.gz
          tar xzf conftest_0.49.1_Linux_x86_64.tar.gz
          sudo mv conftest /usr/local/bin/
          conftest --version

      - name: Run self-test suite
        run: cargo run -p xtask -- selftest

      - name: Upload artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: selftest-artifacts
          path: |
            docs/feature_status.md
            .llm/bundle/*.md
            target/junit/*.xml
          if-no-files-found: warn
```

### Matrix Testing (Multiple Rust Versions)

For libraries that need to support multiple Rust versions:

```yaml
name: XTask Matrix Tests
on:
  push:
    branches: [main, develop]
  pull_request:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  test:
    strategy:
      matrix:
        rust: [stable, beta, nightly, 1.89.0]  # Adjust MSRV as needed
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Run xtask checks
        run: cargo run -p xtask -- check
```

### Custom xtask Commands

If you've extended xtask with custom commands:

```yaml
name: Custom Validation
on:
  push:
    branches: [main]
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      # Run custom xtask commands
      - name: Validate database migrations
        run: cargo run -p xtask -- validate-migrations

      - name: Check API compatibility
        run: cargo run -p xtask -- check-api-compat

      - name: Generate deployment artifacts
        run: cargo run -p xtask -- bundle deployment
```

---

## Required Status Checks

### GitHub Branch Protection

To ensure code quality, configure these required status checks in your repository settings:

**Settings → Branches → Branch protection rules → Add rule**

For `main` branch:

#### Greenfield (Template Users)

Required checks:
- ✅ `selftest` - Core validation suite
- ✅ `rust-lints` (from ci-lints.yml) - Code quality
- ✅ `bdd` (from ci-gherkin.yml) - Acceptance tests
- ✅ `policy-tests` (optional, if using policies)

#### Brownfield (Library Users)

Minimum required checks:
- ✅ `check` - Basic validation (fmt, clippy, tests)

Recommended additional checks:
- ✅ `bdd` - If using BDD acceptance tests
- ✅ `policy` - If using Rego policies
- ✅ `selftest` - For comprehensive validation

### Configuration Steps

1. **Navigate to branch protection:**
   ```
   Repository → Settings → Branches → Add branch protection rule
   ```

2. **Configure the rule:**
   - Branch name pattern: `main` (or your default branch)
   - ☑ Require status checks to pass before merging
   - ☑ Require branches to be up to date before merging

3. **Select status checks:**
   - Search for and select the job names listed above
   - For template users: Select all core workflows
   - For library users: Select at minimum the `check` workflow

4. **Additional recommended settings:**
   - ☑ Require a pull request before merging
   - ☑ Require approvals: 1+
   - ☑ Dismiss stale pull request approvals when new commits are pushed
   - ☑ Require review from Code Owners (if you have a CODEOWNERS file)
   - ☑ Require linear history
   - ☑ Do not allow bypassing the above settings

### Verifying Status Checks

After configuration, verify by:

1. Creating a test branch
2. Making a trivial change
3. Opening a pull request
4. Confirming all required checks run automatically
5. Checking that merge is blocked until checks pass

---

## Estimated CI Run Times

Based on typical repository size and complexity:

### Greenfield (Template Users)

| Workflow | Cold Cache | Warm Cache | Frequency |
|----------|-----------|-----------|-----------|
| **selftest.yml** | ~25-30 min | ~5-8 min | Every PR/push |
| **policy-test.yml** | ~5-8 min | ~2-3 min | Policy changes only |
| **ci-lints.yml** | ~15-20 min | ~3-5 min | Every PR |
| **ci-gherkin.yml** | ~10-15 min | ~3-4 min | Every PR |
| **ci-ac.yml** | ~8-12 min | ~2-3 min | AC changes |
| **ci-security.yml** | ~5-8 min | ~2-3 min | Every PR |
| **ci-coverage.yml** | ~20-25 min | ~8-10 min | Push to main |

**Total for typical PR:** ~30 min (cold) or ~8 min (warm cache)

### Brownfield (Library Users)

| Workflow | Cold Cache | Warm Cache | Notes |
|----------|-----------|-----------|-------|
| **check** | ~10-15 min | ~3-5 min | fmt + clippy + tests |
| **bdd** | ~8-12 min | ~2-4 min | If using BDD |
| **policy** | ~5-8 min | ~2-3 min | If using policies |
| **selftest** | ~20-25 min | ~5-8 min | Comprehensive |

**Minimal setup:** ~15 min (cold) or ~5 min (warm cache)

### Optimization Tips

1. **Use caching effectively:**
   ```yaml
   - uses: Swatinem/rust-cache@v2
   ```

2. **Enable sccache for faster Rust compilation:**
   ```yaml
   - name: Enable sccache
     run: |
       echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
       echo "SCCACHE_GHA_ENABLED=1" >> $GITHUB_ENV
   ```

3. **Use concurrency groups to cancel outdated runs:**
   ```yaml
   concurrency:
     group: ${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: true
   ```

4. **Path filters to skip irrelevant runs:**
   ```yaml
   on:
     pull_request:
       paths-ignore: ['**/*.md', 'docs/**']
   ```

5. **Split long workflows into parallel jobs:**
   ```yaml
   jobs:
     fmt:
       # ...
     clippy:
       # ...
     test:
       # ...
   ```

### Performance Benchmarks

**Template with sccache enabled:**
- First run: ~25-30 minutes
- Subsequent runs (same PR): ~5-8 minutes
- After dependency update: ~10-15 minutes

**Brownfield minimal setup:**
- First run: ~10-15 minutes
- Subsequent runs: ~3-5 minutes

**Factors affecting run time:**
- Repository size
- Number of dependencies
- Complexity of tests
- Cache hit rate
- Runner hardware (GitHub-hosted vs self-hosted)

---

## Troubleshooting

### Common Issues

1. **"conftest not found" in policy-test**
   - Ensure conftest is installed in your environment
   - For Nix users: Already included in devShell
   - For others: Install via your package manager or download from [conftest releases](https://github.com/open-policy-agent/conftest/releases)

2. **Cache misses slowing down builds**
   - Verify `Swatinem/rust-cache@v2` is configured
   - Check cache key validity
   - Consider using sccache for additional speedup

3. **Workflow timeouts**
   - Increase timeout-minutes if legitimate (e.g., large test suite)
   - Check for hanging tests or infinite loops
   - Consider splitting into parallel jobs

4. **Artifacts not uploaded**
   - Verify paths are correct
   - Check if commands actually generate expected files
   - Use `if-no-files-found: warn` for optional artifacts

### Getting Help

- Check [troubleshooting guide](./how-to/troubleshooting.md)
- Review workflow logs in GitHub Actions tab
- See [xtask documentation](./reference/xtask-commands.md)
- File an issue with workflow logs attached

---

## Next Steps

- [Set up branch protection](./BRANCH-PROTECTION-SETUP.md)
- [Configure deployment workflows](./how-to/deploy-to-production.md)
- [Add custom xtask commands](./how-to/extend-xtask.md)
- [Integrate with other CI systems](./how-to/integrate-other-ci.md)
