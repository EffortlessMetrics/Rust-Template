# GitHub Actions Workflows

This directory contains CI/CD workflows for automated testing, validation, and deployment.

## Core Workflows

### Primary Validation

#### `selftest.yml` - Complete Self-Test Suite
**Purpose:** Comprehensive validation of the entire template
**Triggers:** Push to `main`, pull requests
**Runtime:** ~30 min (cold) / ~8 min (warm cache)
**What it tests:**
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Unit and integration tests
- ✅ BDD acceptance tests
- ✅ AC status generation
- ✅ LLM context bundler
- ✅ Policy validation

**Status:** Required for merging PRs

This is the primary validation workflow that ensures all template functionality works correctly.

#### `policy-test.yml` - Fast Policy Validation
**Purpose:** Quick validation of Rego policies
**Triggers:** Changes to `policy/**` files
**Runtime:** ~10 min (cold) / ~3 min (warm cache)
**What it tests:**
- ✅ Ledger policies
- ✅ Feature flag policies
- ✅ Privacy policies
- ✅ Template core policies
- ✅ LLM policies
- ✅ Kubernetes policies

**Status:** Required when policy files change

Fast feedback loop for policy development and validation.

### Code Quality

#### `ci-lints.yml` - Rust Linting
**Purpose:** Validate code quality and style
**Triggers:** Pull requests (except docs)
**Runtime:** ~15-20 min (cold) / ~3-5 min (warm)
**What it checks:**
- ✅ `cargo fmt --check`
- ✅ `cargo clippy --all-targets --all-features`
- ✅ `cargo test --all-features`

**Status:** Required for merging PRs

#### `ci-msrv.yml` - Minimum Supported Rust Version
**Purpose:** Ensure compatibility with MSRV (1.89.0)
**Triggers:** Pull requests
**Runtime:** ~15 min

Validates that code compiles and tests pass on the minimum supported Rust version.

### Acceptance Testing

#### `ci-gherkin.yml` - BDD Acceptance Tests
**Purpose:** Run Gherkin/Cucumber acceptance tests
**Triggers:** Pull requests
**Runtime:** ~10-15 min (cold) / ~3-4 min (warm)
**Artifacts:** JUnit XML reports

Tests user-facing features against acceptance criteria using BDD scenarios.

#### `ci-ac.yml` - Acceptance Criteria Status
**Purpose:** Generate AC status mapping
**Triggers:** Push to main, pull requests
**Runtime:** ~8-12 min
**Artifacts:** `docs/feature_status.md`

Maps Gherkin scenarios to acceptance criteria and generates status documentation.

### Feature & Flag Validation

#### `ci-features.yml` - Feature Flag Validation
**Purpose:** Ensure all feature combinations compile
**Triggers:** Pull requests
**Runtime:** ~15-20 min

Tests various feature flag combinations to catch configuration issues.

#### `ci-flags.yml` - Flag Policy Enforcement
**Purpose:** Validate flag configurations against policies
**Triggers:** Changes to flags
**Runtime:** ~10 min

#### `ci-flags-warn.yml` - Flag Warnings
**Purpose:** Non-blocking warnings for flag best practices
**Triggers:** Changes to flags
**Runtime:** ~5 min

### Security & Privacy

#### `ci-security.yml` - Security Scanning
**Purpose:** Security audit of dependencies and code
**Triggers:** Pull requests, scheduled daily
**Runtime:** ~5-8 min
**Tools:**
- `cargo audit` - Known vulnerabilities
- `cargo deny` - License compliance
- `gitleaks` - Secret scanning

**Status:** Required for merging PRs

#### `ci-privacy.yml` - Privacy Policy Validation
**Purpose:** Validate privacy annotations and policies
**Triggers:** Changes to privacy configs
**Runtime:** ~5 min

Ensures PII handling follows privacy policies.

#### `ci-scope-guard.yml` - Scope Isolation Checks
**Purpose:** Verify scope boundaries are maintained
**Triggers:** Pull requests
**Runtime:** ~5 min

### Governance

#### `ci-governance.yml` - Policy as Code
**Purpose:** Validate governance policies
**Triggers:** Pull requests, scheduled weekly
**Runtime:** ~10-15 min
**Policies checked:**
- Organizational rules
- Compliance requirements
- Best practices
- Resource limits

#### `ci-policy-verify.yml` - Policy Regression Tests
**Purpose:** Ensure policy changes don't break existing validations
**Triggers:** Changes to `policy/**`
**Runtime:** ~10 min

### Events & Monitoring

#### `ci-events.yml` - Event Schema Validation
**Purpose:** Validate event schemas and contracts
**Triggers:** Changes to event definitions
**Runtime:** ~10 min

Ensures event-driven architectures maintain schema compatibility.

### API & Protocol

#### `ci-openapi.yml` - OpenAPI Validation
**Purpose:** Validate OpenAPI specs and generate docs
**Triggers:** Changes to API specs
**Runtime:** ~15 min
**Artifacts:**
- Validated OpenAPI schemas
- Generated API documentation

#### `ci-proto.yml` - Protobuf Validation
**Purpose:** Validate .proto files and check compatibility
**Triggers:** Changes to `*.proto` files
**Runtime:** ~10 min
**Tools:**
- `buf` - Protocol buffer linting and breaking change detection

### Database

#### `ci-db.yml` - Database Migration Tests
**Purpose:** Test database migrations
**Triggers:** Changes to migration files
**Runtime:** ~10 min
**Services:** PostgreSQL test container

Validates migrations are idempotent and don't lose data.

### Infrastructure

#### `ci-nix.yml` - Nix Flake Check
**Purpose:** Validate Nix development environment
**Triggers:** Changes to `flake.nix`
**Runtime:** ~5 min

Ensures the Nix development shell remains functional.

### Documentation

#### `ci-docs.yml` - Documentation Build
**Purpose:** Build and validate documentation
**Triggers:** Changes to docs
**Runtime:** ~5 min

Validates Markdown, builds API docs, checks links.

### Coverage & Quality

#### `ci-coverage.yml` - Code Coverage
**Purpose:** Generate test coverage reports
**Triggers:** Push to main
**Runtime:** ~20-25 min (cold) / ~8-10 min (warm)
**Artifacts:** Coverage reports (lcov, HTML)

**Status:** Informational (not blocking)

### Maintenance

#### `maintenance-pin-actions.yml` - Pin GitHub Actions
**Purpose:** Track updates to GitHub Actions
**Triggers:** Scheduled monthly
**Runtime:** ~2 min

Monitors for updates to GitHub Actions and creates PRs.

### Release

#### `release-sbom-sign.yml` - SBOM & Signing
**Purpose:** Generate SBOM and sign releases
**Triggers:** Release tags (`v*`)
**Runtime:** ~10 min
**Artifacts:**
- Software Bill of Materials (SBOM)
- Signed release artifacts

---

## Template Migration

The existing workflow `ci-template-selftest.yml` is being phased out in favor of the new modular approach:

- **Old:** `ci-template-selftest.yml` (monolithic)
- **New:** `selftest.yml` + `policy-test.yml` (modular)

The new workflows provide:
- ✅ Better separation of concerns
- ✅ Faster feedback (policy changes don't rerun full suite)
- ✅ Clearer naming
- ✅ Easier to understand and maintain

### Migration Timeline

1. ✅ New workflows created: `selftest.yml`, `policy-test.yml`
2. ⏳ Both old and new workflows run in parallel (current state)
3. 🔜 Update branch protection to use new workflow names
4. 🔜 Remove `ci-template-selftest.yml` after validation period

---

## Workflow Architecture

### Concurrency Control

All workflows use concurrency groups to prevent redundant runs:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

This ensures:
- New commits to a PR cancel previous runs
- Resources aren't wasted on outdated code
- Faster feedback loop

### Performance Optimizations

1. **Rust Caching:** `Swatinem/rust-cache@v2`
   - Caches compiled dependencies
   - Significantly reduces build times

2. **sccache:** Shared compilation cache
   ```yaml
   echo "RUSTC_WRAPPER=$(nix develop -c which sccache)" >> $GITHUB_ENV
   ```
   - Caches individual compilation artifacts
   - Works across branches
   - Persists in GitHub Actions cache

3. **Path Filters:** Skip workflows for irrelevant changes
   ```yaml
   paths-ignore: ['**/*.md', 'docs/**']
   ```

4. **Minimal Checkout:** Use shallow clones when possible
   ```yaml
   fetch-depth: 1  # Or 0 for full history when needed
   ```

### Timeout Strategy

| Workflow Type | Timeout | Rationale |
|--------------|---------|-----------|
| Fast checks | 10 min | Linting, policy tests |
| Standard tests | 20-25 min | Unit/integration tests |
| Comprehensive | 30 min | Full self-test suite |
| Coverage | 30 min | Includes instrumentation overhead |

Timeouts prevent hanging jobs from consuming runner time.

### Permissions Model

All workflows use minimal permissions:

```yaml
permissions:
  contents: read
```

Only specific workflows that need write access request it explicitly (e.g., release workflows).

---

## Required Status Checks

Configure these in **Settings → Branches → Branch protection rules:**

### For `main` branch:

**Minimum required:**
- ✅ `selftest` (from selftest.yml)
- ✅ `rust-lints` (from ci-lints.yml)

**Recommended:**
- ✅ `bdd` (from ci-gherkin.yml)
- ✅ `security` (from ci-security.yml)
- ✅ `policy-tests` (from policy-test.yml)

**Optional (based on your needs):**
- `governance` (compliance requirements)
- `openapi` (API-heavy projects)
- `proto` (gRPC services)
- `db` (database changes)

### For feature branches:

Same as `main`, but consider:
- Allowing some checks to be skipped
- Using `paths` filters to run only relevant checks

---

## Running Workflows Locally

### Using Nix (Recommended)

```bash
# Enter dev environment
nix develop

# Run the same commands as CI
cargo run -p xtask -- selftest
cargo run -p xtask -- policy-test
cargo run -p xtask -- check
cargo run -p xtask -- bdd
```

### Without Nix

```bash
# Install dependencies manually (see flake.nix for list)

# Run xtask commands
cargo run -p xtask -- selftest
```

### Using act (Local GitHub Actions)

```bash
# Install act: https://github.com/nektos/act

# Run a specific workflow
act -j selftest

# Run all PR workflows
act pull_request
```

---

## Adding New Workflows

### Template for New Workflow

```yaml
name: Your Workflow Name
on:
  push:
    branches: [main]
  pull_request:
    paths: ['relevant/**', 'paths/**']

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  your-job:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    permissions:
      contents: read
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Run your command
        run: nix develop -c cargo run -p xtask -- your-command
```

### Best Practices

1. **Use descriptive names:** Workflow and job names should be clear
2. **Add path filters:** Skip irrelevant runs
3. **Set appropriate timeouts:** Prevent hanging jobs
4. **Use minimal permissions:** Principle of least privilege
5. **Cache aggressively:** Use rust-cache and sccache
6. **Upload artifacts:** For debugging and reporting
7. **Add to README:** Document what the workflow does

---

## Troubleshooting

### Workflow fails with "command not found"

**Cause:** Tool not available in environment
**Solution:**
- Add to `flake.nix` packages list
- Or install in workflow step before use

### Slow workflow runs

**Cause:** Cache misses or large dependency tree
**Solutions:**
- Verify `rust-cache` is configured
- Enable sccache
- Use path filters to skip unnecessary runs
- Consider splitting into parallel jobs

### Artifacts not uploaded

**Cause:** Files not generated or wrong path
**Solution:**
- Check the command actually creates the files
- Verify paths are correct (relative to workspace root)
- Use `if-no-files-found: warn` for optional artifacts

### Timeout exceeded

**Cause:** Tests hanging or legitimately slow
**Solutions:**
- Increase timeout if needed
- Check for infinite loops or deadlocks
- Add `--test-threads=1` for debugging
- Split tests into parallel jobs

---

## Workflow Dependency Graph

```
selftest.yml (Primary)
├── check (fmt, clippy, tests)
├── bdd (acceptance tests)
├── ac-status (AC mapping)
├── bundle (LLM context)
└── policy-test (Rego validation)

policy-test.yml (Fast)
└── conftest (policy validation)

ci-lints.yml
├── fmt
├── clippy
└── test

ci-security.yml
├── cargo audit
├── cargo deny
└── gitleaks

ci-governance.yml
└── conftest (governance policies)
```

---

## Performance Metrics

Based on typical template usage:

| Workflow | Cold Cache | Warm Cache | Cache Hit Rate |
|----------|-----------|-----------|----------------|
| selftest | 25-30 min | 5-8 min | 85-90% |
| policy-test | 5-8 min | 2-3 min | 90-95% |
| ci-lints | 15-20 min | 3-5 min | 85-90% |
| ci-security | 5-8 min | 2-3 min | 95%+ |

**Total PR validation time:** ~8-10 min (with warm cache)

---

## CI/CD Pipeline Flow

### Pull Request Flow

```
1. Developer opens PR
   ↓
2. Workflows triggered based on changed files
   ↓
3. Parallel execution:
   - selftest (if core files changed)
   - policy-test (if policy/* changed)
   - ci-lints (always)
   - ci-security (always)
   - Specialized workflows (as needed)
   ↓
4. All required checks must pass
   ↓
5. PR approved + checks pass → Merge allowed
```

### Main Branch Flow

```
1. PR merged to main
   ↓
2. Post-merge workflows:
   - selftest (full validation)
   - ci-coverage (optional)
   - AC status generation
   ↓
3. Artifacts uploaded:
   - Coverage reports
   - Feature status
   - JUnit results
```

### Release Flow

```
1. Tag pushed (e.g., v1.0.0)
   ↓
2. Release workflows:
   - release-sbom-sign (SBOM generation)
   - Build artifacts
   - Sign releases
   ↓
3. Publish:
   - GitHub Release
   - Artifacts attached
   - SBOM included
```

---

## External Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cache Action](https://github.com/Swatinem/rust-cache)
- [sccache Documentation](https://github.com/mozilla/sccache)
- [act - Local GitHub Actions](https://github.com/nektos/act)
- [Template CI Examples](../docs/ci-examples.md)

---

## Maintenance

### Regular Tasks

- **Monthly:** Review workflow performance, update actions
- **Quarterly:** Audit required checks, remove obsolete workflows
- **On dependency updates:** Check cache invalidation works correctly

### Updating GitHub Actions

When dependabot or similar opens PRs to update actions:

1. Review changelog for breaking changes
2. Test locally if possible (using act)
3. Merge and monitor first workflow run
4. Rollback if issues detected

---

## Getting Help

- Check workflow logs in Actions tab
- Review [troubleshooting guide](../docs/how-to/troubleshooting.md)
- See [CI examples](../docs/ci-examples.md)
- File issue with workflow logs attached
