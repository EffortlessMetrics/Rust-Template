# Plan: CI Batch 1 - Quick Wins

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** None (CI infrastructure fixes)

## Scope

**Files in scope:**
- `flake.nix` (lines ~30-40) - Add protobuf and cargo-llvm-cov to devshell
- `backstage/docs/index.md` (new file) - Create placeholder documentation
- `.github/workflows/ci-template-selftest.yml` (line ~80) - Fix artifact naming

**CI jobs unblocked:**
- ✅ ci-coverage.yml → coverage job will pass
- ✅ ci-*.yml (multiple) → Any job building workspace will pass
- ✅ ci-docs.yml → backstage doc building will pass
- ✅ ci-template-selftest.yml → artifact uploads won't conflict

## Goals

1. Fix 4 critical CI failures with low-risk additive changes
2. Unblock coverage reporting, full workspace builds, docs building, and artifact uploads
3. Enable immediate deployment without breaking existing functionality
4. Estimated total time: 15 minutes

## Implementation Steps

### 1. Fix Issue #9: protoc missing (HIGH PRIORITY)

**File:** `flake.nix` (lines ~30-40)

**Problem:** Full workspace builds fail when compiling `adapters-grpc` crate because `protoc` is not in Nix devshell

**Fix:** Add `pkgs.protobuf` to buildInputs

**Before:**
```nix
buildInputs = [
  pkgs.rust
  pkgs.just
  pkgs.git
  # ... existing packages ...
  pkgs.cargo-nextest
  pkgs.zlib
];
```

**After:**
```nix
buildInputs = [
  pkgs.rust
  pkgs.just
  pkgs.git
  # ... existing packages ...
  pkgs.cargo-nextest
  pkgs.zlib
  pkgs.protobuf          # Issue #9: protoc for tonic-build (adapters-grpc)
];
```

### 2. Fix Issue #1: cargo-llvm-cov missing (HIGH PRIORITY)

**File:** `flake.nix` (same location as above)

**Problem:** `ci-coverage.yml` coverage job fails because `cargo-llvm-cov` is not in Nix devshell

**Fix:** Add `pkgs.cargo-llvm-cov` to buildInputs

**Add after protobuf:**
```nix
  pkgs.cargo-llvm-cov    # Issue #1: coverage tool for ci-coverage.yml
```

### 3. Fix Issue #2: Backstage docs missing (MEDIUM PRIORITY)

**File:** `backstage/docs/index.md` (new file)

**Problem:** `ci-docs.yml` docs job fails when PR touches backstage/ because `backstage/docs/` directory exists but is empty; mkdocs.yml references missing files

**Fix:** Create placeholder documentation file

**Content:**
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

### 4. Fix Issue #8: Artifact name conflicts (LOW-MEDIUM PRIORITY)

**File:** `.github/workflows/ci-template-selftest.yml` (line ~80)

**Problem:** Matrix jobs don't differentiate artifact names by OS, causing HTTP 409 conflicts when multiple runners complete simultaneously

**Before:**
```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: template-selftest-artifacts
    path: |
      selftest_*.md
      bundle/
```

**After:**
```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: template-selftest-${{ matrix.os }}  # ← differentiate by OS
    path: |
      selftest_*.md
      bundle/
```

## Verification Commands

### After flake.nix changes:

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

### After backstage docs:

```bash
# Verify file exists
ls -la backstage/docs/index.md

# If mkdocs is installed, try building docs
cd backstage && mkdocs build
```

### After artifact fix:

```bash
# Push to PR and watch ci-template-selftest.yml job
# Verify artifacts upload without 409 conflicts
# Check Actions tab → artifact names should be:
#   - template-selftest-ubuntu-latest
#   - template-selftest-macos-latest
#   - etc.
```

## Definition of Done

- [ ] `pkgs.protobuf` added to flake.nix buildInputs
- [ ] `pkgs.cargo-llvm-cov` added to flake.nix buildInputs
- [ ] `nix develop` successfully rebuilds devshell
- [ ] `which protoc` returns path in Nix shell
- [ ] `which cargo-llvm-cov` returns path in Nix shell
- [ ] `cargo build --workspace` succeeds
- [ ] `backstage/docs/index.md` created with placeholder content
- [ ] ci-template-selftest.yml uses `template-selftest-${{ matrix.os }}`
- [ ] Changes committed to branch `chore/ci-batch1-quick-wins`
- [ ] CI jobs turn green after push

## Commit Message

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

## Notes

- **Estimated Effort:** 15 minutes total
- **Risk Level:** Low - all changes are additive, no breaking changes
- **Unblocks:** 5+ CI jobs immediately
- **Branch Strategy:** Create new branch from current working branch
- **Testing Strategy:** Verify locally in Nix shell + watch CI after push
