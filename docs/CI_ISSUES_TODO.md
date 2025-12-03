---
id: GUIDE-CI-ISSUES-TODO
doc_type: reference
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-DEVEX-CONTRACT, REQ-PLT-SECURITY-GOVERNANCE]
---
<!-- doclint:disable orphan-version -->
# CI Infrastructure Issues - GitHub Issue Templates

**Document ID:** `GUIDE-CI-ISSUES-TODO`
**Doc Type:** reference
**Status:** Active
**Last Updated:** 2025-12-02
**Requirements:** None (infrastructure housekeeping)

---

## Purpose

This document catalogs all known CI infrastructure issues that need GitHub issues created for tracking and resolution. These are **global repo issues** not caused by recent changes, but they block CI checks and require systematic cleanup.

Based on analysis from:
- `docs/MAINTAINERS_HANDOVER_2025_12_02.md`
- `PR_ORGANIZATION_PLAN.md`
- GitHub Actions workflow files (`.github/workflows/*.yml`)

---

## Issue 1: Coverage Job - Missing `cargo-llvm-cov`

**Title:** `CI: Coverage job fails - cargo llvm-cov command not found`

**Description:**

The coverage CI job (`.github/workflows/ci-coverage.yml`) fails with:

```
error: no such command: 'llvm-cov'
```

**Root Cause:**

The workflow runs `nix develop -c cargo llvm-cov` but `cargo-llvm-cov` is not installed in the Nix devshell environment defined in `flake.nix`.

**Impact:**
- Coverage job always fails on PRs
- No code coverage reports generated
- CI status shows red even for valid changes

**Proposed Solution:**

Option A: Add `cargo-llvm-cov` to Nix devshell in `flake.nix`:

```nix
devShells = forAllSystems ({ pkgs, rust, ... }: {
  default = pkgs.mkShell {
    packages = [
      rust
      pkgs.cargo-llvm-cov  # Add this line
      pkgs.cargo-audit
      pkgs.cargo-deny
      # ... rest of packages
    ];
  };
});
```

Option B: Disable the coverage job temporarily and add to "future work" backlog

**Labels:**
- `ci`
- `infrastructure`
- `good-first-issue`
- `priority:medium`

**Workaround:**

Comment out or disable the coverage workflow until the tool is available:

```yaml
# .github/workflows/ci-coverage.yml - add 'if: false' to disable
jobs:
  coverage:
    if: false  # Temporarily disabled - missing cargo-llvm-cov
    runs-on: ubuntu-latest
```

---

## Issue 2: Docs Job - Backstage Documentation Missing

**Title:** `CI: Docs job fails - backstage/docs directory missing`

**Description:**

The docs CI job (`.github/workflows/ci-docs.yml`) runs:

```yaml
- run: mkdocs build --strict -f backstage/mkdocs.yml
```

But fails because `backstage/docs/` directory doesn't exist.

**Root Cause:**

The Backstage documentation structure is scaffolded in config (`backstage/mkdocs.yml` exists) but the actual documentation content in `backstage/docs/` has never been created.

**Impact:**
- Docs job always fails on PRs
- Cannot validate Backstage integration documentation
- CI status shows red

**Proposed Solution:**

Option A: Create minimal `backstage/docs/` structure:

```bash
mkdir -p backstage/docs
echo "# Backstage Integration" > backstage/docs/index.md
echo "Documentation coming soon." >> backstage/docs/index.md
```

Option B: Remove the backstage docs check from CI until the plugin is fully implemented:

```yaml
# .github/workflows/ci-docs.yml
- run: mkdocs build --strict -f backstage/mkdocs.yml
  if: hashFiles('backstage/docs/**') != ''  # Only run if docs exist
```

Option C: Disable this workflow step entirely and track backstage docs as future work

**Labels:**
- `ci`
- `documentation`
- `backstage`
- `priority:low`

**Workaround:**

Comment out the backstage docs build step:

```yaml
# .github/workflows/ci-docs.yml
# - run: mkdocs build --strict -f backstage/mkdocs.yml
```

**Related Tasks:**
- `TASK-TPL-BACKSTAGE-002` in `specs/tasks.yaml` (Create Backstage plugin)
- See `docs/how-to/implement-backstage-plugin.md` for implementation guide

---

## Issue 3: Deps Job - cargo-audit doesn't support Cargo.lock v4

**Title:** `CI: cargo-audit fails with Cargo.lock version 4 format`

**Description:**

The deps security job (`.github/workflows/ci-security.yml`) runs `cargo xtask audit`, which calls `cargo-audit`. This fails with:

```
error: Cargo.lock format version 4 is not supported
```

**Root Cause:**

The repository uses Rust Edition 2024 and newer Cargo, which generates `Cargo.lock` with `version = 4` header. The version of `cargo-audit` in the Nix devshell (`pkgs.cargo-audit`) is outdated and only supports up to v3.

**Impact:**
- Dependency security scanning fails
- Cannot detect known vulnerabilities via RustSec advisory database
- CI shows red on security checks

**Proposed Solution:**

Option A: Update Nix to get latest `cargo-audit`:

```bash
nix flake update  # Update flake.lock to get newer pkgs.cargo-audit
```

Option B: Install cargo-audit from source/crates.io in CI:

```yaml
# .github/workflows/ci-security.yml
- run: cargo install cargo-audit --locked
- run: cargo audit
```

Option C: Temporarily skip cargo-audit and only run cargo-deny:

```bash
# crates/xtask/src/commands/audit.rs
if has_cargo_audit && !lock_v4_detected {
    run_cargo_audit()?;
} else {
    eprintln!("⚠️  Skipping cargo-audit (Cargo.lock v4 not supported yet)");
}
```

**Labels:**
- `ci`
- `security`
- `dependencies`
- `priority:high`

**Workaround:**

Modify `crates/xtask/src/commands/audit.rs` to gracefully skip cargo-audit when Cargo.lock v4 is detected, or run it manually with a compatible version outside CI.

---

## Issue 4: Deps Job - cargo-deny doesn't support Edition 2024

**Title:** `CI: cargo-deny fails with Rust Edition 2024 in Cargo.toml`

**Description:**

The deps security job runs `cargo xtask audit`, which calls `cargo-deny`. This fails parsing workspace members with:

```
error: unknown edition '2024' in Cargo.toml
```

**Root Cause:**

The version of `cargo-deny` in the Nix devshell (`pkgs.cargo-deny`) is outdated and doesn't recognize `edition = "2024"` in `Cargo.toml`. Rust Edition 2024 support was added in cargo-deny v0.14.0+.

**Impact:**
- License and source policy checks fail
- Cannot enforce dependency policies
- CI shows red on security checks

**Proposed Solution:**

Option A: Update Nix to get latest `cargo-deny`:

```bash
nix flake update  # Update flake.lock to get newer pkgs.cargo-deny
```

Option B: Install cargo-deny from source/crates.io in CI:

```yaml
# .github/workflows/ci-security.yml
- run: cargo install cargo-deny --locked
- run: cargo deny check
```

Option C: Temporarily downgrade edition to "2021" in workspace Cargo.toml (not recommended)

**Labels:**
- `ci`
- `security`
- `dependencies`
- `priority:high`

**Workaround:**

Same as Issue 3 - either update Nix packages via `nix flake update`, install from crates.io, or temporarily disable the check with a warning.

---

## Issue 5: Secrets Job - GITLEAKS_LICENSE secret not configured

**Title:** `CI: Gitleaks job fails - GITLEAKS_LICENSE secret required`

**Description:**

The secrets scanning job (`.github/workflows/ci-security.yml`) uses `gitleaks/gitleaks-action@v2`, which fails immediately with:

```
Error: GITLEAKS_LICENSE secret not found
```

**Root Cause:**

The Gitleaks GitHub Action requires a license key (free or commercial) to be stored as a repository or organization secret named `GITLEAKS_LICENSE`. This hasn't been configured.

**Impact:**
- Secret scanning always fails
- Cannot detect accidentally committed credentials
- CI shows red on security checks

**Proposed Solution:**

Option A: Get a free Gitleaks license and add it as a GitHub secret:

1. Visit https://gitleaks.io/ and sign up for a free license
2. Copy the license key
3. Add as repository secret:
   - Go to repo Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `GITLEAKS_LICENSE`
   - Value: (paste license key)

Option B: Switch to open-source Gitleaks CLI (no license required):

```yaml
# .github/workflows/ci-security.yml
- name: Install Gitleaks
  run: |
    wget https://github.com/gitleaks/gitleaks/releases/download/v8.18.0/gitleaks_8.18.0_linux_x64.tar.gz
    tar -xzf gitleaks_8.18.0_linux_x64.tar.gz
    sudo mv gitleaks /usr/local/bin/
- name: Run Gitleaks
  run: gitleaks detect --source . --verbose
```

Option C: Use alternative secret scanning (e.g., `trufflesecurity/trufflehog-actions-scan`)

**Labels:**
- `ci`
- `security`
- `secrets`
- `priority:medium`
- `requires-org-admin`

**Workaround:**

Disable the secrets job temporarily:

```yaml
# .github/workflows/ci-security.yml
jobs:
  secrets:
    if: false  # Disabled until GITLEAKS_LICENSE is configured
```

---

## Issue 6: CodeQL Job - Permission Issue Writing Analysis Results

**Title:** `CI: CodeQL analysis succeeds but fails writing status/telemetry`

**Description:**

The CodeQL security analysis job (`.github/workflows/ci-security.yml`) completes the scan successfully but fails at the end with a permission error when trying to write analysis status or telemetry data back to GitHub.

**Root Cause:**

The CodeQL GitHub Action (`github/codeql-action/analyze@v3`) requires `security-events: write` permission, which is configured in the workflow:

```yaml
permissions: { contents: read, security-events: write }
```

However, the error suggests either:
1. A GitHub App permission issue (if using a GitHub App token)
2. The repository settings don't allow CodeQL results to be written
3. Branch protection rules conflict with the action

**Impact:**
- CodeQL analysis results aren't saved to Security tab
- Cannot track security findings over time
- CI shows red even though scan completed

**Proposed Solution:**

Option A: Verify repository settings:

1. Go to repo Settings → Security → Code security and analysis
2. Ensure "CodeQL analysis" is enabled
3. Check that write permissions are allowed

Option B: Check branch protection settings:

1. Go to Settings → Branches → Branch protection rules
2. Ensure CodeQL status checks aren't required before CodeQL can write
3. Look for circular dependencies in required checks

Option C: Use GITHUB_TOKEN explicitly:

```yaml
# .github/workflows/ci-security.yml
- uses: github/codeql-action/analyze@v3
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

Option D: Investigate GitHub App permissions (if applicable)

**Labels:**
- `ci`
- `security`
- `codeql`
- `priority:medium`
- `requires-org-admin`

**Workaround:**

The analysis itself works, so this is a reporting issue. Can continue with partial functionality while investigating root cause. If using GitHub App tokens, ensure the app has `security-events: write` permission.

---

## Issue 7: macOS Nix Installer Job Flakiness

**Title:** `CI: macOS jobs intermittently fail during Nix installation`

**Description:**

CI jobs running on `macos-latest` runners intermittently fail during the Nix installation step (`cachix/install-nix-action@v27` or `DeterminateSystems/nix-installer-action@v9`).

Failure modes:
- Timeout during Nix install (hangs on `/nix` directory creation)
- Permission errors accessing `/nix`
- Network timeouts downloading Nix installer
- Nix daemon fails to start

**Root Cause:**

macOS GitHub Actions runners have inconsistent behavior with:
1. `/nix` directory permissions (sometimes pre-existing from cached runner images)
2. Nix multi-user daemon setup (requires elevated permissions)
3. Network flakiness downloading large Nix store paths
4. Runner image variations (different macOS versions, different cached state)

**Impact:**
- macOS CI jobs are unreliable (10-20% failure rate)
- Tier-1 validation can't complete
- Wastes CI minutes with retries

**Proposed Solution:**

Option A: Add retry logic to Nix install:

```yaml
# .github/workflows/*.yml
- uses: cachix/install-nix-action@v27
  timeout-minutes: 10
  continue-on-error: true
  id: nix_install_1
- uses: cachix/install-nix-action@v27
  if: steps.nix_install_1.outcome == 'failure'
  timeout-minutes: 10
```

Option B: Switch to DeterminateSystems installer (more reliable):

```yaml
- uses: DeterminateSystems/nix-installer-action@v9
- uses: DeterminateSystems/magic-nix-cache-action@v2
```

Option C: Pre-cache Nix install with custom runner image (expensive)

Option D: Clean `/nix` before install:

```yaml
- name: Clean Nix state
  run: sudo rm -rf /nix ~/.nix-* || true
- uses: cachix/install-nix-action@v27
```

**Labels:**
- `ci`
- `infrastructure`
- `macos`
- `flaky-test`
- `priority:medium`

**Workaround:**

Re-run failed jobs manually. Consider Option B (DeterminateSystems installer) which is already used in `ci-supply-chain.yml` and appears more reliable.

**Note:**

Some workflows already use `DeterminateSystems/nix-installer-action@v9` (e.g., `ci-supply-chain.yml`), while others use `cachix/install-nix-action@v27`. Standardizing on one installer across all workflows may improve consistency.

---

## Issue 8: Artifact Upload 409 Conflicts - Duplicate Artifact Names

**Title:** `CI: Artifact upload fails with HTTP 409 - artifact name already exists`

**Description:**

Multiple CI jobs attempt to upload artifacts with the same name in a single workflow run, causing conflicts:

```
Error: Artifact name 'test-results' already exists
HTTP 409 Conflict
```

Affected workflows:
- `.github/workflows/selftest.yml`
- `.github/workflows/ci-ac.yml`
- `.github/workflows/ci-coverage.yml`
- `.github/workflows/ci-template-selftest.yml`

**Root Cause:**

The `actions/upload-artifact@v4` action requires unique artifact names per workflow run. Multiple jobs uploading `test-results`, `junit-xml`, or `cov-json` with the same name causes conflicts.

In matrix jobs (e.g., `ci-template-selftest.yml` runs on `[ubuntu-latest, macos-latest, windows-latest]`), each OS uploads to the same artifact name.

**Impact:**
- Artifact uploads fail intermittently
- Cannot download test results or coverage reports
- CI shows red on otherwise successful runs

**Proposed Solution:**

Option A: Use unique artifact names per job/matrix:

```yaml
# .github/workflows/ci-template-selftest.yml
- uses: actions/upload-artifact@v4
  with:
    name: selftest-results-${{ matrix.os }}  # Include matrix variable
    path: test-results/
```

Option B: Use artifact merge (v4.4.0+ feature):

```yaml
- uses: actions/upload-artifact@v4
  with:
    name: test-results
    path: test-results/
    merge-multiple: true  # Merge if name exists
```

Option C: Overwrite existing artifacts:

```yaml
- uses: actions/upload-artifact@v4
  with:
    name: test-results
    path: test-results/
    overwrite: true
```

Option D: Use separate artifact names per workflow:

- `selftest.yml` → `selftest-results`
- `ci-ac.yml` → `ac-results`
- `ci-coverage.yml` → `coverage-results`
- `ci-template-selftest.yml` → `template-selftest-${{ matrix.os }}`

**Labels:**
- `ci`
- `infrastructure`
- `artifacts`
- `priority:low`
- `good-first-issue`

**Workaround:**

For now, CI will show red for artifact uploads but test execution succeeds. Not a blocker for merging code, but should be fixed for clean CI status.

**Recommended Fix:**

Audit all workflows using `actions/upload-artifact@v4` and ensure:
1. Matrix jobs include matrix variables in artifact names
2. Separate workflows use distinct artifact names
3. Consider using `merge-multiple: true` where appropriate

---

## Issue 9: Protoc Missing - adapters-grpc Compilation Fails

**Title:** `CI: adapters-grpc fails to compile - protoc not found in PATH`

**Description:**

CI jobs that build the full workspace (including `crates/adapters-grpc`) fail with:

```
error: failed to run custom build command for `adapters-grpc`
Could not find `protoc` executable in PATH
```

The `adapters-grpc` crate uses `tonic-build` in its `build.rs`, which requires the Protocol Buffers compiler (`protoc`) to be installed.

**Root Cause:**

The Nix devshell in `flake.nix` does not include `protobuf` (which provides the `protoc` binary). The crate compiles locally for developers who have protoc installed globally, but fails in CI.

**Impact:**
- Workspace-wide builds fail (`cargo build --workspace`)
- Integration tests can't run
- `adapters-grpc` feature is untested in CI

**Proposed Solution:**

Option A: Add protobuf to Nix devshell:

```nix
# flake.nix
devShells = forAllSystems ({ pkgs, rust, ... }: {
  default = pkgs.mkShell {
    packages = [
      rust
      pkgs.protobuf  # Add this line for protoc
      pkgs.cargo-audit
      pkgs.cargo-deny
      # ... rest of packages
    ];
  };
});
```

Option B: Install protoc in CI before build:

```yaml
# .github/workflows/*.yml
- name: Install protoc
  run: |
    sudo apt-get update
    sudo apt-get install -y protobuf-compiler
```

Option C: Make adapters-grpc optional and exclude from default builds:

```toml
# Cargo.toml workspace
default-members = [
  "crates/app-http",
  "crates/business-core",
  # Exclude adapters-grpc from default build
]
```

**Labels:**
- `ci`
- `build`
- `adapters-grpc`
- `dependencies`
- `priority:medium`

**Workaround:**

Exclude `adapters-grpc` from workspace builds temporarily:

```bash
# In CI or locally
cargo build --workspace --exclude adapters-grpc
```

Or install protoc locally:

```bash
# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# macOS
brew install protobuf

# Or via Nix
nix profile install nixpkgs#protobuf
```

**Recommended Fix:**

Option A (add to Nix devshell) is cleanest, as it ensures consistent tooling across all environments. This aligns with the "Nix-first dev env" philosophy (see `docs/adr/0002-nix-first-dev-env.md`).

---

## Summary Table

| Issue | Priority | Blocking CI? | Complexity | Recommended Approach |
|-------|----------|--------------|------------|---------------------|
| 1. cargo-llvm-cov missing | Medium | Yes | Low | Add to flake.nix |
| 2. Backstage docs missing | Low | Yes | Low | Create placeholder docs or disable check |
| 3. cargo-audit Cargo.lock v4 | High | Yes | Medium | Update Nix flake or install from crates.io |
| 4. cargo-deny Edition 2024 | High | Yes | Medium | Update Nix flake or install from crates.io |
| 5. GITLEAKS_LICENSE secret | Medium | Yes | Low | Get license and add secret |
| 6. CodeQL permissions | Medium | No | Medium | Investigate repo/app settings |
| 7. macOS Nix installer flaky | Medium | Intermittent | Medium | Switch to DeterminateSystems installer |
| 8. Artifact upload 409 | Low | No | Low | Use unique artifact names |
| 9. protoc missing | Medium | Yes | Low | Add to flake.nix |

---

## Action Plan

### Phase 1: Quick Wins (Priority: High, Complexity: Low)

1. **Issue 9 (protoc)** - Add `pkgs.protobuf` to `flake.nix` (5 minutes)
2. **Issue 2 (backstage docs)** - Create placeholder `backstage/docs/index.md` or disable check (5 minutes)
3. **Issue 8 (artifacts)** - Audit workflows and rename artifacts (15 minutes)

### Phase 2: Tooling Updates (Priority: High, Complexity: Medium)

4. **Issue 3 & 4 (cargo-audit/deny)** - Run `nix flake update` to get compatible versions (10 minutes)
5. **Issue 1 (llvm-cov)** - Add to flake.nix or temporarily disable coverage (5 minutes)

### Phase 3: External Dependencies (Priority: Medium, Requires Admin)

6. **Issue 5 (Gitleaks license)** - Obtain license and configure secret (requires org admin)
7. **Issue 6 (CodeQL)** - Investigate repository/app settings (requires org admin)

### Phase 4: Infrastructure Hardening (Priority: Medium, Complexity: Medium)

8. **Issue 7 (macOS flakiness)** - Standardize on DeterminateSystems Nix installer (30 minutes)

---

## How to Create GitHub Issues

For each issue above, create a GitHub issue with:

1. **Title** from the issue section
2. **Description** (copy the full Description + Root Cause + Impact sections)
3. **Labels** as specified
4. **Assignee**: Leave unassigned or assign to CI/infrastructure owner
5. **Milestone**: "CI Hardening" or "v3.4.0-prep"
6. **Body template**:

```markdown
## Problem

[Copy Description + Root Cause + Impact]

## Proposed Solution

[Copy Proposed Solution section with all options]

## Workaround

[Copy Workaround section]

## References

- `docs/CI_ISSUES_TODO.md` - Full context
- `docs/MAINTAINERS_HANDOVER_2025_12_02.md` - Handover notes
- Related workflow: [link to .github/workflows/*.yml]

## Acceptance Criteria

- [ ] CI job passes without errors
- [ ] No workarounds needed
- [ ] Documented in CHANGELOG.md
- [ ] Verified in at least one full CI run
```

---

## Notes

- These issues are **not blockers** for merging PRs with correct functionality (e.g., PR-1 environment fixes, PR-3 documentation).
- CI red status from these issues should be explicitly called out in PR descriptions.
- Track as separate "CI Hardening" epic, not as part of template feature development.
- Some issues require **org-level admin permissions** (secrets, CodeQL config, branch protection).

---

## References

- **Source Analysis:**
  - `docs/MAINTAINERS_HANDOVER_2025_12_02.md` (Section 5: CI failures)
  - `PR_ORGANIZATION_PLAN.md` (Known issues section)
  - Workflow files in `.github/workflows/`

- **Related ADRs:**
  - `docs/adr/0002-nix-first-dev-env.md` - Nix devshell philosophy
  - `docs/adr/0007-dependency-security-health.md` - cargo-audit/deny rationale

- **Related Docs:**
  - `docs/TROUBLESHOOTING.md` - Known environment issues
  - `docs/reference/ci-workflows.md` - CI architecture
  - `flake.nix` - Nix package definitions

---

## Maintenance

This document should be updated when:

- Issues are resolved (mark as ✅ RESOLVED with date and PR/commit)
- New CI issues are discovered
- Workarounds change
- Tooling is updated (e.g., new Nix packages available)

Last audit: 2025-12-02 (based on kernel v3.3.6 state)
