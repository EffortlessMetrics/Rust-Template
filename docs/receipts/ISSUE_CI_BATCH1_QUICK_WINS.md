# CI Batch 1: Quick Wins (High Priority, Low Risk)

## Overview

This batch addresses 4 critical CI failures that can be fixed quickly with low risk. All fixes are additive (no breaking changes) and can be deployed immediately.

**Estimated Total Time:** 15 minutes
**Risk Level:** Low
**Unblocks:** 5+ CI jobs (coverage, full builds, backstage docs, artifact uploads)

## Issues Included

### Issue #9: protoc missing (HIGH PRIORITY)

- **Blocker:** Full workspace builds fail when compiling `adapters-grpc` crate
- **Root Cause:** `protoc` not in Nix devshell; `adapters-grpc/build.rs` uses `tonic-build`
- **Impact:** Any CI job building workspace fails
- **Fix:** Add `pkgs.protobuf` to flake.nix line ~36
- **Time:** 2 minutes

### Issue #1: cargo-llvm-cov missing (HIGH PRIORITY)

- **Blocker:** `ci-coverage.yml` coverage job fails
- **Root Cause:** `cargo-llvm-cov` not in Nix devshell
- **Impact:** All PR coverage reports fail
- **Fix:** Add `pkgs.cargo-llvm-cov` to flake.nix line ~36
- **Time:** 2 minutes

### Issue #2: Backstage docs missing (MEDIUM PRIORITY)

- **Blocker:** `ci-docs.yml` docs job fails when PR touches backstage/
- **Root Cause:** `backstage/docs/` directory exists but is empty; mkdocs.yml references missing files
- **Impact:** Doc building fails on backstage changes
- **Fix:** Create placeholder `backstage/docs/index.md`
- **Time:** 2 minutes

### Issue #8: Artifact name conflicts (LOW-MEDIUM PRIORITY)

- **Blocker:** ci-template-selftest.yml artifact uploads fail with HTTP 409
- **Root Cause:** Matrix jobs don't differentiate artifact names by OS
- **Impact:** Artifact uploads fail intermittently when multiple runners complete
- **Fix:** Rename artifact to `template-selftest-${{ matrix.os }}`
- **Time:** 2 minutes

## Files to Change

### 1. `flake.nix` (lines ~30-40)

**Current packages list:**

```nix
buildInputs = [
  pkgs.rust
  pkgs.just
  pkgs.git
  pkgs.curl
  pkgs.jq
  pkgs.yq-go
  pkgs.nodejs
  pkgs.python3
  pkgs.gitleaks
  pkgs.conftest
  pkgs.kubectl
  pkgs.kustomize
  pkgs.cargo-audit
  pkgs.cargo-deny
  pkgs.cargo-nextest
  pkgs.zlib
];
```

**Add these two lines:**

```nix
  pkgs.protobuf          # Issue #9: protoc for tonic-build (adapters-grpc)
  pkgs.cargo-llvm-cov    # Issue #1: coverage tool for ci-coverage.yml
```

### 2. `backstage/docs/index.md` (new file)

Create with minimal placeholder content:

```markdown
# Backstage Documentation

This is a placeholder for Backstage-specific documentation.

## Architecture

(To be documented)

## Integration

(To be documented)

## Development

(To be documented)
```

### 3. `.github/workflows/ci-template-selftest.yml` (line ~80)

**Current artifact upload:**

```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: template-selftest-artifacts
    path: |
      selftest_*.md
      bundle/
```

**Fix:** Add matrix variable to name:

```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: template-selftest-${{ matrix.os }}  # ← differentiate by OS
    path: |
      selftest_*.md
      bundle/
```

## Verification Steps

### After flake.nix changes

```bash
# Update Nix environment
nix develop

# Verify protoc available
which protoc
protoc --version

# Verify cargo-llvm-cov available
which cargo-llvm-cov
cargo llvm-cov --version

# Try full workspace build (should now work)
cargo build --workspace

# Try coverage (should now work)
cargo llvm-cov --workspace
```

### After backstage docs

```bash
# Verify file exists
ls -la backstage/docs/index.md

# If mkdocs is installed, try building docs
cd backstage && mkdocs build
```

### After artifact fix

- Push to PR
- Watch ci-template-selftest.yml job
- Verify artifacts upload without 409 conflicts
- Check Actions tab → artifact names should be `template-selftest-ubuntu-latest`, `template-selftest-macos-latest`, etc.

## CI Jobs Unblocked

✅ **ci-coverage.yml** → coverage job will pass
✅ **ci-*.yml** (multiple) → Any job building workspace will pass
✅ **ci-docs.yml** → backstage doc building will pass
✅ **ci-template-selftest.yml** → artifact uploads won't conflict

## Success Criteria

- [ ] `pkgs.protobuf` added to flake.nix
- [ ] `pkgs.cargo-llvm-cov` added to flake.nix
- [ ] `nix develop` successfully rebuilds devshell
- [ ] `which protoc` returns path
- [ ] `which cargo-llvm-cov` returns path
- [ ] `cargo build --workspace` succeeds
- [ ] `backstage/docs/index.md` created with placeholder content
- [ ] ci-template-selftest.yml uses `template-selftest-${{ matrix.os }}`
- [ ] Push changes to branch
- [ ] Verify CI jobs turn green

## Branch Strategy

**Target branch:** `chore/ci-batch1-quick-wins` (new branch from `chore/env-and-handover-2025-12-02`)

**Commit message:**

```
fix(ci): Add protoc, cargo-llvm-cov, backstage docs, and fix artifact naming

- Add pkgs.protobuf to flake.nix (fixes adapters-grpc builds)
- Add pkgs.cargo-llvm-cov to flake.nix (fixes coverage CI)
- Create backstage/docs/index.md placeholder (fixes doc builds)
- Differentiate ci-template-selftest artifact names by OS (fixes upload conflicts)

Fixes: Issue #1, #2, #8, #9
Related: CI_ISSUES_TODO.md Batch 1
```

## Follow-up PRs

After this batch merges:
- **Batch 2:** Update Nix packages (`nix flake update`) for cargo-audit/deny v4 support
- **Batch 3:** Standardize Nix installer across 22 workflows
- **Batch 4:** Gitleaks license + CodeQL permissions (admin tasks)

## Related Documentation

- `docs/CI_ISSUES_TODO.md` - Full CI issues catalog
- `docs/MAINTAINERS_HANDOVER_2025_12_02.md` - Handover notes
- `.github/workflows/README.md` - Workflow architecture
- `docs/reference/ci-workflows.md` - Detailed CI docs

## Labels

`ci`, `high-priority`, `quick-win`, `nix`, `devshell`, `backstage`, `artifacts`
