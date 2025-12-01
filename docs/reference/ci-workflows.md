---
id: REF-CI-WORKFLOWS-001
title: "CI Workflows Reference Guide"
doc_type: reference
version: 3.3.4
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-DEVEX-CONTRACT
  - REQ-PLT-ONBOARDING
  - REQ-PLT-SECURITY-GOVERNANCE
  - REQ-PLT-RELEASE-SAFETY
  - REQ-PLT-DOCS-CONSISTENCY
acs:
  - AC-PLT-014
  - AC-PLT-015
  - AC-PLT-016
  - AC-PLT-019
  - AC-PLT-020
adrs:
  - ADR-0002
  - ADR-0005
  - ADR-0017
last_updated: 2025-11-26
---

# CI Workflows Reference

This document provides a comprehensive reference for all CI workflows in this Rust-as-Spec platform cell.

---

## Table of Contents

- [Overview](#overview)
- [Tier System](#tier-system)
- [Workflow Inventory](#workflow-inventory)
- [Required Status Checks](#required-status-checks)
- [Debugging CI Failures](#debugging-ci-failures)
- [Adding New Workflows](#adding-new-workflows)
- [Performance and Optimization](#performance-and-optimization)

---

## Overview

The CI system is designed around **governance enforcement** and **fast feedback**. All workflows validate that code changes conform to the platform's contracts, specs, and policies.

### Core Principles

1. **Governance as Code:** All workflows enforce platform contracts via `cargo xtask selftest` and policy tests
2. **Fast Feedback:** Path filters and concurrency control prevent redundant runs
3. **Tier-Based Validation:** Tier 1 (Linux/macOS + Nix) is canonical; Tier 2 (Windows native) is supported but limited
4. **Minimal Permissions:** All workflows use least-privilege security model
5. **Cached Builds:** Rust-cache and sccache reduce build times by 70-85%

### Workflow Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Pull Request                         │
└─────────────────┬───────────────────────────────────────┘
                  │
        ┌─────────┴─────────────────┐
        │  Path-based Triggering    │
        │  (Skip irrelevant checks) │
        └─────────┬─────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐    ┌───▼───┐    ┌───▼────┐
│ Tier 1│    │Security│   │Governance│
│ Gates │    │ Checks │   │  Checks  │
└───┬───┘    └───┬───┘    └───┬────┘
    │            │            │
    └────────────┴────────────┘
                 │
         All Required Checks
              Must Pass
                 │
                 ▼
            Merge Allowed
```

---

## Tier System

The platform uses a tiered validation approach based on the environment:

### Tier 1: Linux/macOS + Nix (Canonical)

**Environment:** Ubuntu/macOS with Nix development shell
**Status:** Full `cargo xtask selftest` runs (all 7 steps)
**Runtime:** 10-20 minutes with warm cache
**Use Case:** Required gate for merging to `main`

**What runs:**
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Unit and integration tests
- ✅ BDD acceptance tests
- ✅ AC status generation
- ✅ LLM context bundler
- ✅ Policy validation (Rego/OPA)

**Workflow:** `tier1-selftest.yml`

### Tier 2: Windows Native (Supported)

**Environment:** Windows without Nix
**Status:** Basic validation only (`cargo xtask check` + workspace tests)
**Runtime:** 15-30 minutes
**Use Case:** Developer iteration, not merge gate

**What runs:**
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Unit and integration tests (excluding BDD)
- ❌ BDD acceptance tests (not run)
- ❌ Policy validation (not run)
- ❌ Full selftest (not run)

**Workflow:** `ci-template-selftest.yml` (Windows job only)

**Why the difference?**
- Native Windows has file locking issues that make full selftest slow (2+ hours)
- WSL2 with Nix is recommended for Windows developers who need full validation
- See `docs/reference/platform-support.md` for details

---

## Workflow Inventory

### Primary Validation Workflows

#### `tier1-selftest.yml` - Main Merge Gate
**Purpose:** Full governance validation in canonical Tier 1 environment
**Triggers:** Push to `main`, pull requests to `main`
**Runtime:** 10-20 minutes (warm cache)
**Blocks Merges:** ✅ Yes (required check)

**What it validates:**
1. Code quality (fmt, clippy, tests)
2. BDD acceptance tests
3. AC status mapping
4. LLM bundler functionality
5. Policy-as-code (Rego)
6. DevEx contract (required commands)
7. Graph invariants

**When it fails:**
- Code doesn't compile or has lint errors
- Tests are broken
- BDD scenarios don't match ACs
- Policies are violated (e.g., AC has no tests)
- Graph integrity is broken (e.g., orphaned requirements)

**How to debug:**
```bash
# Run locally in Nix shell
nix develop
cargo xtask selftest

# For specific step failures
cargo xtask policy-test     # If policy step fails
cargo xtask bdd             # If BDD step fails
cargo xtask ac-status       # If AC mapping fails
```

---

#### `ci-template-selftest.yml` - Multi-Platform Validation
**Purpose:** Validate on Linux, macOS, and Windows (all tiers)
**Triggers:** Push to `main` or `claude/**`, all pull requests
**Runtime:** 10-30 minutes depending on platform
**Blocks Merges:** ✅ Yes (Linux/macOS jobs required)

**Matrix Strategy:**
- **Linux + Nix (Tier 1):** Full selftest
- **macOS + Nix (Tier 1):** Full selftest
- **Windows native (Tier 2):** Basic validation only

**Artifacts Generated:**
- `docs/feature_status.md` - AC status mapping
- `.llm/bundle/implement_ac.md` - LLM context bundle
- `target/junit/acceptance.xml` - BDD test results

**Clean Git State Enforcement:**
After selftest runs, the workflow verifies that generated artifacts (like `feature_status.md`) are committed. If the git tree is dirty, the workflow fails with:
```
❌ Git tree is dirty after running selftest.
💡 Please run 'cargo xtask selftest' locally and commit the changes.
```

**When it fails:**
- Same reasons as `tier1-selftest.yml`
- Additionally: Generated docs are out of sync (must be committed)

---

### Code Quality Workflows

#### `ci-lints.yml` - Rust Code Quality
**Purpose:** Fast linting and formatting checks
**Triggers:** Pull requests (excluding docs changes)
**Runtime:** 3-5 minutes (warm cache)
**Blocks Merges:** ✅ Yes (required check)

**What it runs:**
```bash
cargo xtask check  # Runs fmt, clippy, and tests
```

**Path Filters:**
```yaml
paths-ignore: ['**/*.md', 'docs/**']
```

**When it fails:**
- Code is not formatted with `rustfmt`
- Clippy warnings (all warnings are errors)
- Unit tests fail

**How to fix:**
```bash
# Format code
cargo fmt

# Fix clippy warnings
cargo clippy --fix --allow-dirty

# Run tests
cargo test
```

---

#### `ci-msrv.yml` - Minimum Supported Rust Version
**Purpose:** Ensure compatibility with MSRV (1.89.0)
**Triggers:** Pull requests
**Runtime:** 10-15 minutes
**Blocks Merges:** ❌ No (informational)

**What it validates:**
- Code compiles on Rust 1.89.0
- All tests pass on MSRV

**When it fails:**
- Code uses features from newer Rust versions
- Dependencies require newer Rust

**How to fix:**
- Avoid unstable features or bump MSRV in `Cargo.toml`
- Check dependency requirements

---

#### `ci-coverage.yml` - Test Coverage
**Purpose:** Measure test coverage and enforce floor (60%)
**Triggers:** Pull requests (changes to code/tests/specs)
**Runtime:** 15-25 minutes
**Blocks Merges:** ✅ Yes (must meet 60% floor)

**Coverage Floor:** 60% line coverage

**What it does:**
```bash
cargo llvm-cov --workspace --json --output-path cov.json
# Then enforces PCT >= 60
```

**Artifacts:**
- `cov.json` - Coverage data in JSON format

**When it fails:**
- Line coverage drops below 60%

**How to fix:**
- Add tests for uncovered code
- Or justify why coverage dropped and adjust floor (requires approval)

---

### Governance and Spec Validation

#### `ci-governance.yml` - Governance PR Checks
**Purpose:** Validate spec changes and PR titles
**Triggers:** Pull requests (changes to `specs/**`)
**Runtime:** 5-10 minutes
**Blocks Merges:** ⚠️ Informational (comments on PR)

**What it does:**
1. Lists impacted spec files in PR comment
2. Checks PR title includes AC/US/FT prefix (e.g., `AC-123`, `US-456`)
3. Posts hint if title doesn't follow convention

**PR Title Convention:**
```
AC-123: Implement platform status endpoint
US-456: Add user authentication
FT-789: Feature toggle for beta features
```

**Bypass:** Add label `skip-title-check` to suppress title validation

**When it comments:**
- PR changes files in `specs/` directory
- PR title doesn't include AC/US/FT prefix

---

#### `ci-gherkin.yml` - BDD Scenario Validation
**Purpose:** Lint Gherkin files and enforce AC tagging
**Triggers:** Changes to `specs/features/**` or `spec_ledger.yaml`
**Runtime:** 3-5 minutes
**Blocks Merges:** ✅ Yes (required check)

**What it validates:**
1. Gherkin syntax (using `gherkin-lint`)
2. Every Scenario has exactly one `@AC-####` tag

**Example Valid Scenario:**
```gherkin
@AC-TPL-001
Scenario: Health endpoint returns 200 OK
  Given the service is running
  When I request GET /health
  Then the response status should be 200
```

**Example Invalid Scenario:**
```gherkin
# ❌ No AC tag
Scenario: Health endpoint returns 200 OK
  Given the service is running
  When I request GET /health
  Then the response status should be 200

# ❌ Multiple AC tags
@AC-TPL-001 @AC-TPL-002
Scenario: Health endpoint returns 200 OK
  Given the service is running
  When I request GET /health
  Then the response status should be 200
```

**When it fails:**
- Gherkin syntax errors
- Scenario has zero or multiple `@AC-####` tags

**How to fix:**
```bash
# Lint locally
npx gherkin-lint -c .gherkin-lintrc specs/features

# Ensure each scenario has exactly one @AC-#### tag
```

---

#### `ci-ac.yml` - Acceptance Criteria Status
**Purpose:** Run BDD tests and generate AC status report
**Triggers:** Changes to features, specs, or acceptance tests
**Runtime:** 15-25 minutes
**Blocks Merges:** ✅ Yes (BDD tests must pass)

**What it does:**
1. Runs all BDD acceptance tests via `cargo xtask bdd`
2. Generates `docs/feature_status.md` with AC coverage
3. Validates AC coverage policy (all kernel ACs must have tests)

**Two Jobs:**
- `bdd`: Run BDD tests and generate status
- `policy`: Validate AC coverage via Rego policy

**Artifacts:**
- `docs/feature_status.md` - AC status mapping
- JUnit XML reports

**When it fails:**
- BDD tests fail (step definitions broken or scenarios fail)
- AC coverage policy fails (kernel AC has no tests)

**How to fix:**
```bash
# Run BDD tests locally
cargo xtask bdd

# Check AC coverage
cargo xtask ac-coverage

# Generate scenarios for missing AC
cargo xtask ac-suggest-scenarios AC-PLT-XXX
```

---

#### `ci-policy-verify.yml` - Policy-as-Code Enforcement
**Purpose:** Validate Rego policies and test data
**Triggers:** Changes to `policy/**`
**Runtime:** 5-10 minutes
**Blocks Merges:** ✅ Yes (policies must be valid)

**What it validates:**
- Rego policy syntax
- Policy test cases pass
- Policy coverage is complete

**Policies enforced:**
- `ledger.rego` - Spec ledger integrity
- `template_core.rego` - Template contracts
- `privacy.rego` - PII handling
- `k8s.rego` - Kubernetes security

**When it fails:**
- Rego syntax error
- Policy test fails
- Policy logic is incorrect

**How to fix:**
```bash
# Test policies locally
cargo xtask policy-test

# Or use conftest directly
conftest test -p policy/ <input-file>
```

---

### Security Workflows

#### `ci-security.yml` - Security Scanning
**Purpose:** Scan for vulnerabilities, secrets, and license issues
**Triggers:** Pull requests (excluding docs), scheduled daily
**Runtime:** 5-10 minutes
**Blocks Merges:** ✅ Yes (required check)

**Three Jobs:**

1. **CodeQL Analysis**
   - Static analysis for security vulnerabilities
   - Language: Rust
   - Uploads results to GitHub Security tab

2. **Secret Scanning (Gitleaks)**
   - Scans commits for hardcoded secrets
   - Checks API keys, passwords, tokens

3. **Dependency Audit**
   - `cargo xtask audit` - Known CVEs in dependencies
   - Checks against RustSec advisory database

**When it fails:**
- CodeQL finds potential vulnerabilities
- Gitleaks detects hardcoded secrets
- Cargo audit finds CVEs in dependencies

**How to fix:**
```bash
# Run audit locally
cargo xtask audit

# View detailed report
cargo audit

# Update vulnerable dependencies
cargo update -p <dependency-name>

# If false positive, add to audit.toml
```

---

#### `ci-supply-chain.yml` - SBOM and Provenance
**Purpose:** Generate SBOM and sign release artifacts
**Triggers:** Git tags matching `v*.*.*` (e.g., `v3.3.3`)
**Runtime:** 15-20 minutes
**Blocks Merges:** N/A (release workflow)

**What it generates:**
1. **SBOM (Software Bill of Materials):** SPDX JSON format
2. **Build Provenance:** GitHub Attestations API
3. **Source Tarball:** Signed release archive

**Artifacts:**
- `rust-template-v3.3.3.tar.gz` - Source archive
- `rust-template-v3.3.3-sbom.spdx.json` - SBOM
- Provenance attestations (GitHub Attestations API)

**Permissions Required:**
- `contents: write` - Write to releases
- `id-token: write` - OIDC token for Sigstore
- `attestations: write` - GitHub Attestations API

**How to use:**
```bash
# Create release
git tag v3.3.3
git push origin v3.3.3

# Workflow automatically:
# - Builds release artifacts
# - Generates SBOM
# - Signs with Sigstore
# - Uploads to GitHub Release
```

---

### API and Contract Validation

#### `ci-openapi.yml` - OpenAPI Schema Validation
**Purpose:** Validate OpenAPI specs and detect breaking changes
**Triggers:** Changes to API spec files
**Runtime:** 10-15 minutes
**Blocks Merges:** ✅ Yes (if API files changed)

**What it validates:**
- OpenAPI schema syntax
- Breaking changes in API contracts
- Schema consistency

**Tools Used:**
- `openapi-generator-cli` - Validation
- `oasdiff` - Breaking change detection

**When it fails:**
- OpenAPI spec has syntax errors
- Breaking changes detected in stable API

**How to fix:**
```bash
# Validate locally
npx @openapitools/openapi-generator-cli validate -i specs/openapi.yaml

# Check for breaking changes
npx oasdiff breaking specs/openapi-old.yaml specs/openapi-new.yaml
```

---

#### `ci-proto.yml` - Protobuf Validation
**Purpose:** Validate `.proto` files and check compatibility
**Triggers:** Changes to `*.proto` files
**Runtime:** 5-10 minutes
**Blocks Merges:** ✅ Yes (if proto files changed)

**What it validates:**
- Protobuf syntax
- Breaking changes
- Wire compatibility

**Tools Used:**
- `buf` - Protobuf linting and breaking change detection

**When it fails:**
- Proto syntax errors
- Breaking changes in stable messages
- Field numbering conflicts

**How to fix:**
```bash
# Lint locally
buf lint

# Check breaking changes
buf breaking --against main
```

---

### Specialized Checks

#### `ci-db.yml` - Database Migration Tests
**Purpose:** Test database migrations for correctness
**Triggers:** Changes to migration files
**Runtime:** 8-12 minutes
**Blocks Merges:** ✅ Yes (if migrations changed)

**What it validates:**
- Migrations apply cleanly
- Migrations are idempotent
- No data loss in migrations

**Test Environment:**
- PostgreSQL container
- Test database with seed data

**When it fails:**
- Migration syntax error
- Migration causes data loss
- Non-idempotent migration

---

#### `ci-docs.yml` - Documentation Build
**Purpose:** Build and validate documentation
**Triggers:** Changes to `docs/**`
**Runtime:** 3-5 minutes
**Blocks Merges:** ✅ Yes (if docs changed)

**What it validates:**
- Markdown syntax
- Link validity
- Documentation builds successfully

**When it fails:**
- Broken links
- Markdown syntax errors
- Missing referenced files

---

#### `ci-nix.yml` - Nix Flake Validation
**Purpose:** Validate Nix development environment
**Triggers:** Changes to `flake.nix` or `flake.lock`
**Runtime:** 3-5 minutes
**Blocks Merges:** ✅ Yes (if Nix files changed)

**What it validates:**
- Flake evaluates correctly
- Development shell builds
- All packages are available

**When it fails:**
- Flake syntax error
- Package not found
- Version conflicts

**How to fix:**
```bash
# Check flake locally
nix flake check

# Update flake inputs
nix flake update
```

---

#### `ci-features.yml` - Feature Flag Matrix
**Purpose:** Test all feature flag combinations
**Triggers:** Pull requests
**Runtime:** 15-20 minutes
**Blocks Merges:** ✅ Yes (required check)

**What it tests:**
- Code compiles with all feature combinations
- Tests pass with each feature enabled/disabled

**Matrix:**
- Default features
- No default features
- All features
- Individual features

**When it fails:**
- Code doesn't compile with certain feature combinations
- Tests fail with specific features

---

#### `ci-flags.yml` - Flag Policy Enforcement
**Purpose:** Validate feature flag configurations
**Triggers:** Changes to flags
**Runtime:** 5-10 minutes
**Blocks Merges:** ✅ Yes (required check)

**What it validates:**
- Flag configurations are valid
- Flags follow naming conventions
- No orphaned flags

---

#### `ci-flags-warn.yml` - Flag Best Practices (Informational)
**Purpose:** Non-blocking warnings for flag usage
**Triggers:** Changes to flags
**Runtime:** 3-5 minutes
**Blocks Merges:** ❌ No (informational only)

**What it checks:**
- Flag documentation
- Deprecation notices
- Usage recommendations

---

#### `ci-events.yml` - Event Schema Validation
**Purpose:** Validate event schemas and contracts
**Triggers:** Changes to event definitions
**Runtime:** 8-10 minutes
**Blocks Merges:** ✅ Yes (if events changed)

**What it validates:**
- Event schema syntax
- Schema compatibility
- Event versioning

---

#### `ci-privacy.yml` - Privacy Compliance
**Purpose:** Validate privacy annotations and policies
**Triggers:** Changes to privacy configs
**Runtime:** 3-5 minutes
**Blocks Merges:** ✅ Yes (if privacy files changed)

**What it validates:**
- PII is properly annotated
- Privacy policies are enforced
- No PII in logs

---

#### `ci-scope-guard.yml` - Scope Isolation
**Purpose:** Verify scope boundaries are maintained
**Triggers:** Pull requests
**Runtime:** 3-5 minutes
**Blocks Merges:** ✅ Yes (required check)

**What it validates:**
- Module boundaries respected
- No cross-scope leaks
- Clean architecture maintained

---

### Maintenance and Automation

#### `maintenance-pin-actions.yml` - Action Version Management
**Purpose:** Track updates to GitHub Actions
**Triggers:** Scheduled monthly
**Runtime:** 2-3 minutes
**Blocks Merges:** N/A (maintenance workflow)

**What it does:**
- Monitors for GitHub Actions updates
- Creates PRs for version bumps
- Ensures security patches applied

---

## Required Status Checks

Configure these in **Settings → Branches → Branch protection rules** for the `main` branch:

### Critical (Must Pass for Merge)

These checks are **mandatory** and will block merges if they fail:

- ✅ **`selftest`** (from `tier1-selftest.yml`) - Full governance validation
- ✅ **`rust-lints`** (from `ci-lints.yml`) - Code quality gate
- ✅ **`bdd`** (from `ci-ac.yml`) - BDD acceptance tests
- ✅ **`codeql`** (from `ci-security.yml`) - Security analysis
- ✅ **`secrets`** (from `ci-security.yml`) - Secret scanning
- ✅ **`deps`** (from `ci-security.yml`) - Dependency audit

### Recommended (Strongly Encouraged)

These checks provide valuable feedback but might be made optional for some workflows:

- ⚠️ **`policy-tests`** (from `policy-test.yml`) - Policy validation
- ⚠️ **`coverage`** (from `ci-coverage.yml`) - Test coverage floor

### Conditional (Based on Changes)

These checks only run when specific file types are modified:

- 🔀 **`lint`** (from `ci-gherkin.yml`) - Gherkin validation (when `specs/features/**` changes)
- 🔀 **`openapi`** (from `ci-openapi.yml`) - API schema validation (when API specs change)
- 🔀 **`proto`** (from `ci-proto.yml`) - Protobuf validation (when `.proto` files change)
- 🔀 **`db`** (from `ci-db.yml`) - Database migrations (when migrations change)

### Setting Up Branch Protection

```bash
# Using GitHub CLI
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["selftest","rust-lints","bdd","codeql","secrets","deps"]}'
```

Or configure via web UI:
1. Go to **Settings → Branches**
2. Add rule for `main` branch
3. Enable "Require status checks to pass before merging"
4. Select required checks from list above

---

## Debugging CI Failures

### General Debugging Strategy

```
1. Identify which workflow failed
   ↓
2. Click into failed job to see logs
   ↓
3. Identify which step failed
   ↓
4. Reproduce locally using same command
   ↓
5. Fix issue and push
```

### Common Failure Scenarios

#### Scenario: `tier1-selftest.yml` fails at "Policy tests" step

**Symptom:**
```
[5/7] Running policy tests...
  ✗ Policy tests failed
FAIL - AC-TPL-001 has no tests
```

**Diagnosis:**
```bash
# Run policy tests locally
cargo xtask policy-test

# Check which AC is failing
cargo xtask ac-coverage | grep "❌"
```

**Fix:**
```bash
# Add tests array to AC in spec_ledger.yaml
# Example:
# acs:
#   - id: AC-TPL-001
#     description: "Health endpoint returns 200 OK"
#     tests:
#       - type: bdd
#         tag: "@AC-TPL-001"

# Or generate BDD scenarios
cargo xtask ac-suggest-scenarios AC-TPL-001
```

---

#### Scenario: `ci-lints.yml` fails with clippy warnings

**Symptom:**
```
error: unused variable: `foo`
  --> src/main.rs:10:9
   |
10 |     let foo = 42;
   |         ^^^ help: if this is intentional, prefix it with an underscore: `_foo`
```

**Diagnosis:**
Clippy treats all warnings as errors in CI.

**Fix:**
```bash
# Fix automatically where possible
cargo clippy --fix --allow-dirty

# Or fix manually and suppress specific warnings if justified
#[allow(unused_variables)]
let foo = 42;
```

---

#### Scenario: `ci-security.yml` fails with cargo audit

**Symptom:**
```
error: 1 vulnerability found!
crate:  openssl
version: 0.10.45
warning: openssl vulnerable to CVE-2023-XXXXX
```

**Diagnosis:**
Dependency has known vulnerability.

**Fix:**
```bash
# Update vulnerable dependency
cargo update -p openssl

# If update not available, add to audit.toml to suppress (requires justification)
```

---

#### Scenario: `ci-gherkin.yml` fails with "Scenario must have exactly one @AC-#### tag"

**Symptom:**
```
specs/features/health.feature: Scenario at line 15 must have exactly one @AC-#### tag (found 0: [])
```

**Diagnosis:**
BDD scenario is missing `@AC-####` tag.

**Fix:**
```gherkin
# Before (invalid)
Scenario: Health endpoint returns 200 OK
  Given the service is running
  When I request GET /health
  Then the response status should be 200

# After (valid)
@AC-TPL-001
Scenario: Health endpoint returns 200 OK
  Given the service is running
  When I request GET /health
  Then the response status should be 200
```

---

#### Scenario: `ci-ac.yml` fails with "Git tree is dirty"

**Symptom:**
```
❌ Git tree is dirty after running selftest. The following files were modified:
  docs/feature_status.md

💡 Please run 'cargo xtask selftest' locally and commit the changes.
```

**Diagnosis:**
Generated documentation is out of sync.

**Fix:**
```bash
# Run selftest locally
cargo xtask selftest

# Commit generated docs
git add docs/feature_status.md
git commit -m "chore: update feature status docs"
git push
```

---

#### Scenario: `ci-coverage.yml` fails with "coverage below floor"

**Symptom:**
```
coverage=58%
test: 58 >= 60 failed
```

**Diagnosis:**
Line coverage dropped below 60% floor.

**Fix:**
```bash
# Generate coverage report locally
cargo llvm-cov --html

# Open coverage report
open target/llvm-cov/html/index.html

# Add tests for uncovered code
# Or justify why coverage dropped (requires approval to adjust floor)
```

---

### Debugging Tips

#### View Logs in Detail

Click into failed job → Click on failed step → Expand log

#### Download Artifacts

Many workflows upload artifacts that can help debug:
- `feature-status` - AC status report
- `cov-json` - Coverage data
- `junit-results` - Test results

Download from workflow run page → "Artifacts" section

#### Reproduce Locally

Most workflows use `nix develop -c cargo xtask <command>`, so you can reproduce exactly:

```bash
# Enter Nix shell
nix develop

# Run the same command as CI
cargo xtask selftest
cargo xtask bdd
cargo xtask check
```

#### Check Workflow Concurrency

If a workflow is stuck "Pending":
- Check if another run of the same workflow is in progress
- CI uses concurrency groups to cancel old runs when new commits are pushed

#### Check Path Filters

If a workflow didn't run when expected:
- Check `paths` and `paths-ignore` filters in workflow YAML
- Some workflows only run when specific files change

---

## Adding New Workflows

### Workflow Template

```yaml
name: Your Workflow Name

on:
  push:
    branches: [main]
  pull_request:
    paths:
      - 'relevant/**'
      - '.github/workflows/your-workflow.yml'

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

      - name: Run your command
        run: nix develop -c cargo xtask your-command

      - name: Show sccache stats
        if: always()
        run: nix develop -c sccache --show-stats || true
```

### Best Practices for New Workflows

1. **Use descriptive names:** Workflow and job names should clearly describe what they do
2. **Add path filters:** Skip runs for irrelevant changes using `paths` or `paths-ignore`
3. **Set appropriate timeouts:** Prevent hanging jobs (typical: 10-25 minutes)
4. **Use minimal permissions:** Start with `contents: read`, add more only if needed
5. **Enable caching:** Use `rust-cache` and `sccache` for faster builds
6. **Upload artifacts:** For debugging and reporting (use `actions/upload-artifact@v4`)
7. **Add to doc index:** Update `specs/doc_index.yaml` with new workflow entry
8. **Document in this file:** Add to "Workflow Inventory" section above

### Checklist for New Workflows

- [ ] Workflow YAML is syntactically valid
- [ ] Concurrency group configured
- [ ] Timeout set (no infinite runs)
- [ ] Permissions are minimal
- [ ] Caching enabled (rust-cache + sccache)
- [ ] Path filters configured (if applicable)
- [ ] Artifacts uploaded (if applicable)
- [ ] Entry added to `specs/doc_index.yaml`
- [ ] Documentation added to this file
- [ ] Tested locally with `act` or similar
- [ ] Linked to relevant REQs/ACs in doc index

---

## Performance and Optimization

### Cache Strategy

The CI system uses multiple caching layers for optimal performance:

#### 1. Rust Dependency Cache (`rust-cache`)

**Action:** `Swatinem/rust-cache@v2`

**What it caches:**
- `~/.cargo/registry` - Crate registry index
- `~/.cargo/git` - Git dependencies
- `target/` - Compiled dependencies

**Cache Key:**
- Rust toolchain version
- `Cargo.lock` hash
- Workflow file hash

**Effect:** 70-80% faster builds on cache hit

**Configuration:**
```yaml
- uses: Swatinem/rust-cache@v2
```

#### 2. Compilation Cache (`sccache`)

**Tool:** Mozilla sccache (shared compilation cache)

**What it caches:**
- Individual compilation artifacts
- Object files
- Incremental compilation data

**Storage:** GitHub Actions cache

**Effect:** 50-60% faster incremental builds

**Configuration:**
```yaml
- name: Enable sccache
  run: |
    echo "RUSTC_WRAPPER=$(nix develop -c which sccache)" >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=1" >> $GITHUB_ENV
    echo "CARGO_INCREMENTAL=0" >> $GITHUB_ENV
    nix develop -c sccache --start-server || true
```

#### 3. Nix Store Cache

**Action:** `cachix/install-nix-action@v27`

**What it caches:**
- Nix store paths
- Development shell environment
- System dependencies

**Effect:** 90% faster Nix setup (5s instead of 5min)

**Configuration:**
```yaml
- uses: cachix/install-nix-action@v27
```

### Performance Metrics

Based on typical runs:

| Workflow | Cold Cache | Warm Cache | Speedup |
|----------|-----------|-----------|---------|
| `tier1-selftest` | 25-30 min | 10-20 min | 2-3x |
| `ci-lints` | 15-20 min | 3-5 min | 4-5x |
| `ci-ac` | 20-25 min | 8-12 min | 2-3x |
| `ci-security` | 8-10 min | 3-5 min | 2-3x |
| `ci-coverage` | 20-25 min | 10-15 min | 2x |

**Cache Hit Rate:** Typically 85-95% on PR branches

### Optimization Tips

#### For Workflow Authors

1. **Use path filters aggressively:**
   ```yaml
   paths-ignore: ['**/*.md', 'docs/**', '*.txt']
   ```

2. **Cancel stale runs:**
   ```yaml
   concurrency:
     group: ${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: true
   ```

3. **Split long jobs into parallel jobs:**
   ```yaml
   jobs:
     test-1:
       run: cargo test -p crate-1
     test-2:
       run: cargo test -p crate-2
   ```

4. **Use conditional steps:**
   ```yaml
   - name: Upload coverage
     if: github.event_name == 'push'
   ```

#### For Developers

1. **Keep PRs focused:** Smaller PRs = fewer workflows triggered
2. **Use draft PRs:** Mark PRs as draft to skip some checks
3. **Fix locally first:** Run `cargo xtask check` before pushing
4. **Commit generated docs:** Run `cargo xtask selftest` and commit artifacts

---

## Further Reading

- **Platform Support Guide:** `docs/reference/platform-support.md`
- **Agent Guide:** `docs/AGENT_GUIDE.md` (section 4: Validation and Verification)
- **Selective Testing:** `docs/SELECTIVE_TESTING.md`
- **xtask Commands:** `docs/reference/xtask-commands.md`
- **Existing Workflow README:** `.github/workflows/README.md`
- **ADR-0017:** `docs/adr/0017-tier1-selftest-gate.md` (Tier-1 gate rationale)

---

## Summary

### Quick Reference: When Each Workflow Runs

| Workflow | Trigger | Blocks PR? | Runtime |
|----------|---------|-----------|---------|
| `tier1-selftest.yml` | Push to main, PRs | ✅ Yes | 10-20 min |
| `ci-template-selftest.yml` | All PRs | ✅ Yes | 10-30 min |
| `ci-lints.yml` | PRs (code changes) | ✅ Yes | 3-5 min |
| `ci-ac.yml` | PRs (specs/features) | ✅ Yes | 15-25 min |
| `ci-security.yml` | PRs, daily | ✅ Yes | 5-10 min |
| `ci-governance.yml` | PRs (specs) | ❌ No | 5-10 min |
| `ci-gherkin.yml` | PRs (features) | ✅ Yes | 3-5 min |
| `ci-coverage.yml` | PRs (code) | ✅ Yes | 15-25 min |
| `ci-supply-chain.yml` | Release tags | N/A | 15-20 min |
| Others | Conditional | Varies | Varies |

### When a PR Can Merge

A pull request can be merged when:
1. ✅ All required status checks pass (see "Required Status Checks")
2. ✅ At least one approval from a maintainer
3. ✅ Git tree is clean (no uncommitted generated files)
4. ✅ PR title follows convention (or `skip-title-check` label)
5. ✅ No merge conflicts with `main`

### Quick Debug Commands

```bash
# Reproduce most CI failures locally
nix develop
cargo xtask selftest        # Full validation
cargo xtask check           # Fast checks
cargo xtask bdd             # BDD tests only
cargo xtask policy-test     # Policy validation
cargo xtask ac-coverage     # Check AC coverage

# Check what changed
cargo xtask test-changed    # Run only affected tests

# Fix common issues
cargo fmt                   # Format code
cargo clippy --fix          # Fix clippy warnings
cargo xtask audit           # Check for vulnerabilities
```

---

**Document Maintenance:**
- Update this document when adding/removing workflows
- Keep "Workflow Inventory" section in sync with actual workflows
- Update performance metrics quarterly based on actual CI data
- Review "Required Status Checks" when governance policies change
