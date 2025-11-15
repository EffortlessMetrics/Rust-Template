# Branch Protection Setup for Rust-Template

This document provides step-by-step instructions for configuring branch protection on the `main` branch of the Rust-Template repository.

## Profile: Standard

The template repository uses the **Standard** profile (see `docs/reference/branch-protection-profiles.md`) because:
- It's a production-ready template used by real services
- Template contracts must remain stable
- We want strong governance without being overly strict

## Required Status Checks

Configure these checks as **required** before merging to `main`:

1. **Template Self-Test** - Validates template integrity
2. **Lints** - Code quality (fmt, clippy, tests)
3. **Nix Flake Check** - Nix environment validity
4. **MSRV** - Minimum Supported Rust Version compatibility
5. **Policy Verification** - Rego policies pass
6. **Privacy** - Privacy spec compliance
7. **Features** - Feature tracking compliance
8. **Flags** - Feature flags compliance
9. **ACs** - Acceptance criteria tracking
10. **Events** - Event schema compliance

## Setup Steps

### 1. Navigate to Branch Protection Settings

1. Go to https://github.com/EffortlessMetrics/Rust-Template
2. Click **Settings** → **Branches**
3. Click **Add branch protection rule**

### 2. Configure Rule Pattern

- **Branch name pattern**: `main`

### 3. Enable Core Protections

Check these boxes:

- ✅ **Require a pull request before merging**
  - Require approvals: **1**
  - ✅ Dismiss stale pull request approvals when new commits are pushed
  - ✅ Require review from Code Owners (if CODEOWNERS file exists)

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - **Search for and select these status checks:**
    - `Template Self-Test`
    - `Lints`
    - `Nix Flake Check`
    - `MSRV`
    - `Policy Verification`
    - `Privacy`
    - `Features`
    - `Flags`
    - `ACs`
    - `Events`

- ✅ **Require conversation resolution before merging**

- ❌ **Do not allow bypassing the above settings** (admins can bypass, but shouldn't)

- ❌ **Do not allow force pushes** (keep unchecked/disabled)

- ❌ **Do not allow deletions** (keep unchecked/disabled)

### 4. Save Rule

Click **Create** or **Save changes**

## Verification

After setting up branch protection:

1. Create a test branch: `git checkout -b test-branch-protection`
2. Make a trivial change (e.g., update this file)
3. Push and open a PR
4. Verify that all required checks appear and run
5. Close the test PR without merging

## Maintenance

### Adding New Checks

When adding new CI workflows that should block merges:

1. Add the check to this document
2. Update `.github/workflows/` with the new check
3. Test the check passes on a feature branch
4. Add the check name to branch protection settings
5. Announce the change to contributors

### Removing Checks

Only remove required checks if:
- The check is permanently deprecated
- You've verified no active PRs depend on it
- You've announced the change with 1-week notice

## Troubleshooting

**Q: A required check doesn't appear in the status check list**

A: The check must run at least once on any branch before GitHub makes it available. Push a commit to any branch to trigger the check, then it will appear in branch protection settings.

**Q: Can I temporarily bypass a check?**

A: Admins can bypass checks, but this should be **very rare** and only for emergencies (e.g., fixing a security vulnerability). Always fix the underlying issue and restore the check ASAP.

**Q: What if a check is flaky?**

A: Disable the check in branch protection immediately, fix the flakiness, then re-enable. Never leave flaky checks required.

## References

- Full profile documentation: `docs/reference/branch-protection-profiles.md`
- GitHub docs: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches
