# ADR-0017: Tier-1 Selftest as Required Gate on Main Branch

**Status**: Accepted
**Date**: 2025-11-26
**Authors**: Steven Zimmerman
**Related ACs**: AC-PLT-015, AC-PLT-016, AC-PLT-019, AC-PLT-020
**Related ADRs**: ADR-0002 (Nix-first dev environment), ADR-0005 (Selftest as single gate)

---

## Context

Software projects face a chronic challenge: local validation diverges from CI, leading to:

- **"Works on my machine" failures**: PRs pass locally but fail in CI
- **Toolchain drift**: Developers use different versions of Rust, clippy, rustfmt
- **Platform-specific issues**: Windows file locking, Unix-only assumptions
- **Non-deterministic builds**: Subtle differences in environment variables, system libraries
- **Blocked merges**: Changes land on main that break the build for others

Traditional approaches have limitations:

1. **"Just run tests before commit"**: No enforcement, relies on discipline
2. **Pre-commit hooks**: Optional, developers can skip with `--no-verify`
3. **CI-only validation**: Slow feedback loop (10+ minutes vs local validation)
4. **Platform-specific CI**: Multiple validation tiers create confusion about "what is canonical?"

We need:

- **Single source of truth** for "what must pass before merge"
- **Hermetic environment** that matches CI exactly
- **Platform independence** that works on Linux, macOS, WSL2
- **Mandatory enforcement** via branch protection

The repository already has two validation environments:

- **Tier 1 (Nix devshell)**: Hermetic, reproducible, matches CI exactly
- **Tier 2 (native Windows)**: Fast iteration but has platform limitations (file locking on steps 7-8)

**The question:** Which tier should CI enforce as the gate for merging to main?

---

## Decision

We adopt **Tier-1 selftest as the mandatory quality gate** for the main branch.

### Core Principle

**"If Tier-1 selftest passes in CI, the change is safe to merge."**

All PRs targeting `main` MUST pass the `tier1-selftest` GitHub Actions job before merge.

### Definition of Tier-1

**Environment:**
- Linux (Ubuntu-latest) + Nix devshell
- Hermetic build environment (no system dependency leakage)
- Exact versions pinned in `flake.nix` (Rust, conftest, cargo-binstall, etc.)

**Validation:**
```bash
nix develop --command cargo xtask selftest
```

This runs all 7 selftest phases:
1. Core checks (fmt, clippy, tests)
2. BDD acceptance tests
3. AC status mapping & ADR references
4. LLM context bundler
5. Policy tests (conftest/OPA)
6. DevEx contract validation
7. Graph invariants

**Success criteria:**
- All 7 phases complete without errors
- No warnings escalated to errors (clippy `-D warnings`)
- Clean exit code (0)

### CI Enforcement

**GitHub Actions workflow:**

```yaml
# .github/workflows/tier1-selftest.yml
name: tier1-selftest

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  selftest:
    runs-on: ubuntu-latest
    timeout-minutes: 40
    permissions:
      contents: read

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v27
        with:
          nix_path: nixpkgs=channel:nixos-24.05

      - name: Run kernel selftest (Tier 1)
        run: |
          nix develop --command cargo xtask selftest
```

**Branch protection:**
- Repository settings → Branches → `main`
- Require status checks to pass before merging: **`tier1-selftest / selftest`** ✅
- Require branches to be up to date before merging: **enabled**

**What this means:**
- No PR can merge to `main` with red selftest
- No force-push override (unless admin)
- No "merge anyway" for urgent fixes (selftest must pass first)

### Tier-2 Status

**Tier-2 (native Windows) is informational, not gating.**

Why:
- Windows has non-deterministic file locking issues (antivirus, file explorer)
- Steps 7-8 may fail with `os error 5` even when code is correct
- Blocking on Tier-2 would create false negatives

**Developer workflow:**
- **Daily iteration**: Use Tier-2 (native Windows) for fast feedback
- **Pre-PR validation**: Use WSL2 + Nix (Tier-1) for canonical check
- **Merge decision**: Only Tier-1 CI result matters

### Rationale

**Why Tier-1 is canonical:**

1. **Hermetic environment**: Nix ensures no system dependency leakage (exact Rust version, exact clippy version)
2. **Reproducible**: Same environment on every developer's machine, same as CI
3. **Platform-independent**: Works identically on Linux, macOS, WSL2
4. **No file locking issues**: Unix semantics (Windows file locking doesn't block Tier-1)
5. **Fast feedback in CI**: ~5-10 minutes for full selftest (vs 30+ minutes for non-hermetic matrix)

**Why not Tier-2:**

1. **Platform-specific failures**: Windows file locking is not a code quality issue
2. **Non-deterministic**: Same code can pass/fail based on antivirus behavior
3. **No CI benefit**: Tier-2 doesn't run in CI (GitHub Actions is Linux)

**Why not both:**

1. **Confusion**: Two tiers create ambiguity ("which one is ground truth?")
2. **Maintenance burden**: Must align two environments instead of one
3. **Slower CI**: Running both tiers doubles CI time with no quality gain

---

## Consequences

### Positive

- **Single source of truth**: Developers know "Tier-1 selftest green = safe to merge"
- **No environment drift**: Nix pins all tools (Rust, conftest, cargo-binstall), CI matches local exactly
- **Fast feedback**: Developers can run Tier-1 locally before push (~5-10 min vs waiting for CI)
- **Cross-platform confidence**: Tier-1 works identically on Linux, macOS, WSL2
- **No false positives**: Tier-1 doesn't suffer from Windows file locking or antivirus interference
- **Clear escalation**: If Tier-1 passes locally but fails in CI, it's a real environment issue (not platform quirk)

### Negative

- **Nix requirement for canonical validation**: Developers must install Nix or use WSL2 for final checks
  - Mitigation: Nix is quick to install (~5 min), WSL2 available on all modern Windows
- **Windows developers need two environments**: Tier-2 for iteration, Tier-1 for final validation
  - Mitigation: WSL2 integration is seamless (VS Code Remote-WSL, Docker Desktop WSL2 backend)
- **Adoption barrier**: Teams unfamiliar with Nix may resist
  - Mitigation: `docs/reference/platform-support.md` documents full onboarding, native fallback exists

### Neutral

- **Tier-2 still supported**: Native Windows workflow remains documented and functional for daily iteration
- **CI time unchanged**: Tier-1 is already the only selftest job in CI (no slowdown)
- **Local iteration unaffected**: Developers can still use `cargo xtask check` for fast loops

---

## Enforcement

### Automated

**Branch protection:**
```yaml
# Required status checks (GitHub repo settings)
- tier1-selftest / selftest  # Must pass
```

**CI configuration:**
- `.github/workflows/tier1-selftest.yml` runs on `push` and `pull_request` to `main`
- Timeout: 40 minutes (ample headroom for slow CI runners)
- Permissions: `contents: read` (minimal access)

**Failure modes:**
- If Nix install fails → CI fails → merge blocked
- If any selftest phase fails → CI fails → merge blocked
- If timeout exceeded → CI fails → merge blocked

### Manual

**Developer workflow:**

1. **Before creating PR:**
   ```bash
   # Enter Tier-1 environment
   nix develop

   # Run selftest
   cargo xtask selftest

   # If green, push
   git push origin feature-branch
   ```

2. **If selftest fails locally:**
   - Fix the failure (tests, clippy, fmt, etc.)
   - Re-run selftest
   - Do not push red changes

3. **If selftest passes locally but fails in CI:**
   - Check if Nix flake is up-to-date (`nix flake update`)
   - Check if commits are up-to-date with `main`
   - Verify local Nix environment matches CI (`nix develop --rebuild`)
   - If still failing, investigate CI logs (likely environment issue)

**Review expectations:**

- Reviewers can assume "CI green = selftest passed in Tier-1"
- If PR shows red CI, reviewer should not approve until green
- No exceptions for "urgent fixes" (selftest must pass first)

### Detection

**If a bad change lands on main:**

1. **How it could happen:**
   - Admin force-merge (bypasses branch protection)
   - Branch protection accidentally disabled
   - Selftest bug (false positive in CI)

2. **Detection:**
   - Next PR will fail Tier-1 selftest (cumulative validation)
   - Developers pulling `main` will see local selftest fail
   - Scheduled CI runs (if enabled) will alert

3. **Recovery:**
   - Revert the bad commit
   - Run Tier-1 selftest on revert
   - Merge revert PR (must pass selftest)
   - Original author fixes issue in new PR

---

## Operational Notes

### Developer Onboarding

**First-time setup (Tier-1):**

```bash
# Install Nix (one-time, 5 minutes)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# Clone repository
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# Enter Tier-1 environment
nix develop

# Verify
cargo xtask doctor
cargo xtask selftest
```

**Expected outcome:**
- All 7 selftest phases pass ✅
- Environment matches CI exactly
- Developer can now work with confidence

**Estimated time:**
- Nix install: 5 minutes
- First `nix develop`: 10 minutes (downloads packages)
- Subsequent `nix develop`: instant (cached)

### Windows-Specific Workflow

**Recommended path for Windows developers:**

```powershell
# One-time: Install WSL2 (10 minutes)
wsl --install
wsl --set-default-version 2

# Inside WSL2 (Tier-1 environment)
wsl
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install --determinate

# Clone inside WSL2 (not /mnt/c/)
cd ~
git clone https://github.com/EffortlessMetrics/Rust-Template.git
cd Rust-Template

# Enter Tier-1
nix develop

# Validate
cargo xtask selftest
```

**Hybrid workflow (fast iteration + canonical validation):**

```powershell
# Daily work: native Windows (Tier-2, fast)
cargo run -p app-http
cargo test

# Before PR: WSL2 (Tier-1, canonical)
wsl -e bash -c "cd ~/Rust-Template && nix develop -c cargo xtask selftest"
```

### CI Optimization

**Caching strategy:**

```yaml
# Future improvement: Use cachix for Nix store
- uses: cachix/cachix-action@v12
  with:
    name: rust-template
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
```

**Benefits:**
- First CI run: ~10 minutes (cold cache)
- Subsequent runs: ~5 minutes (warm cache)
- No change to validation guarantee (still hermetic)

### Escape Hatches

**When Tier-1 CI is broken (infrastructure issue):**

1. **Symptom**: Nix install fails, GitHub Actions outage, etc.
2. **Diagnosis**: Check if issue is code-related or CI-related
3. **Decision**:
   - If code-related → fix code, re-run Tier-1
   - If CI-related → wait for infrastructure recovery or contact maintainer
4. **No bypass**: Do not merge until Tier-1 passes (ensures main stays green)

**Emergency hotfix (production incident):**

1. **If Tier-1 selftest blocks critical fix:**
   - Diagnose: Is the failure related to the hotfix? If no, consider if the fix is truly critical.
   - Escalate: Admin-level decision (document in ADR addendum)
   - Merge: Force-merge with admin override (bypass branch protection)
   - Remediate: File issue, fix Tier-1 failure in follow-up PR
   - Document: Add note to CHANGELOG explaining override

2. **Preferred path:**
   - Fix Tier-1 failure first (even if urgent)
   - Merge via normal process (selftest passes)
   - No shortcuts

---

## Compliance

### How to verify this decision is followed

**Automated checks:**

1. **Branch protection enabled:**
   ```bash
   # Check via GitHub API
   curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/repos/EffortlessMetrics/Rust-Template/branches/main/protection

   # Expected: required_status_checks includes "tier1-selftest / selftest"
   ```

2. **CI job exists and runs:**
   ```bash
   # Check workflow file
   cat .github/workflows/tier1-selftest.yml

   # Expected: triggers on push/PR to main
   ```

3. **Selftest phases documented:**
   ```bash
   # Run locally
   cargo xtask selftest --help

   # Expected: Lists 7 phases with descriptions
   ```

**Manual audit:**

1. **Review recent PRs:**
   - All merged PRs should show green `tier1-selftest / selftest` check
   - No PRs merged with red selftest (except admin overrides, documented)

2. **Review branch protection settings:**
   - Repository → Settings → Branches → `main`
   - Verify `tier1-selftest / selftest` is required
   - Verify "Require branches to be up to date" is enabled

3. **Review documentation:**
   - `docs/reference/platform-support.md` documents Tier-1 as canonical
   - `CLAUDE.md` instructs agents to use Tier-1 for final validation
   - `README.md` links to platform support and onboarding guides

---

## Migration Path

**For existing projects adopting this pattern:**

1. **Add Nix flake:**
   - Copy `flake.nix` from this template
   - Pin tool versions to match current CI

2. **Create Tier-1 CI workflow:**
   - Copy `.github/workflows/tier1-selftest.yml`
   - Update Nix channel to match flake

3. **Enable branch protection:**
   - Repository → Settings → Branches → `main`
   - Add `tier1-selftest / selftest` to required checks

4. **Communicate to team:**
   - Announce Tier-1 as new canonical validation
   - Provide onboarding guide (Nix install, WSL2 setup)
   - Set deadline (e.g., "all PRs after 2025-12-01 must pass Tier-1")

5. **Soft rollout (optional):**
   - Week 1: Tier-1 CI runs but not required (informational)
   - Week 2: Enable branch protection (enforcement)
   - Week 3+: Tier-1 is canonical

---

## References

- **Related ADRs:**
  - [ADR-0002: Nix-First Development Environment](0002-nix-first-dev-env.md) – Defines Tier-1 environment
  - [ADR-0005: Selftest as the Single Quality Gate](0005-xtask-selftest-single-gate.md) – Defines selftest phases

- **Documentation:**
  - [Platform Support Reference](../reference/platform-support.md) – Tier-1 vs Tier-2 comparison
  - [CLAUDE.md](../../CLAUDE.md) – Agent instructions (use Tier-1 for final validation)
  - [CI Coverage Reference](../reference/ci-coverage.md) – What CI validates

- **Spec Ledger:**
  - `REQ-PLT-DEVEX-CONTRACT` – DevEx flows are spec-backed and CI-enforced
  - `AC-PLT-015` – `cargo xtask selftest` enforces devex contract
  - `AC-PLT-016` – `cargo xtask ci-local` orchestrates full validation

- **External References:**
  - [Nix Flakes](https://nixos.wiki/wiki/Flakes) – Reproducible dev environments
  - [GitHub Branch Protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
  - [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer) – Fast, reliable Nix setup

---

## Summary

**The Decision:**

> All changes to `main` MUST pass Tier-1 selftest (Nix devshell + `cargo xtask selftest`) in CI before merge. No exceptions.

**Why:**

> Tier-1 provides a hermetic, reproducible environment that eliminates platform-specific issues and ensures CI matches local validation exactly.

**Enforcement:**

> GitHub branch protection requires `tier1-selftest / selftest` to pass. Developers can run Tier-1 locally for fast feedback.

**Impact:**

> Tier-1 becomes the single source of truth for "ready to merge." Tier-2 (native Windows) remains supported for fast iteration but is not gating.

**Escape Hatch:**

> Admin override exists for emergencies but requires documentation and follow-up remediation.
