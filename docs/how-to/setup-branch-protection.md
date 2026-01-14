---
id: GUIDE-TPL-BRANCH-PROTECTION-001
title: Setup GitHub Branch Protection
doc_type: how-to
status: published
audience: developers, maintainers, platform-engineers
tags: [security, github, branch-protection, governance, ci]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-RELEASE-SAFETY, REQ-PLT-SECURITY-GOVERNANCE]
acs: [AC-PLT-011, AC-PLT-012, AC-PLT-013]
adrs: [ADR-0005, ADR-0006, ADR-0017]
last_updated: 2025-11-26
---

# How-to: Setup GitHub Branch Protection

**Time:** 10 minutes
**Prerequisites:** Repository admin access, `gh` CLI installed (optional)

This guide shows you how to configure GitHub branch protection rules to enforce governance checks and prevent accidental direct pushes to main.

---

## Why Branch Protection Matters

For the Rust-as-Spec template, branch protection is a critical enforcement layer:

1. **Prevents bypass of selftest** - All changes must pass `cargo xtask selftest` via CI before merging
2. **Ensures peer review** - Changes are reviewed before reaching main
3. **Protects release tags** - Prevents force-pushes that could break version history
4. **Enforces governance** - Makes the contracts and CI checks the final authority

Without branch protection, developers can:
- Push directly to main, bypassing CI
- Merge PRs with failing tests
- Force-push and rewrite history
- Skip pre-commit hooks

With branch protection configured, the template's governance is enforced at the platform level.

---

## Recommended Settings

For a Rust-as-Spec platform cell, configure these rules on the `main` branch:

### Required Status Checks

- ✅ **Require status checks to pass before merging**
  - `tier1-selftest` (the comprehensive governance check)
  - `ci-security` (cargo audit and deny)
  - `ci-docs` (documentation consistency)
  - `ci-policy-verify` (OPA policy checks)

### Branch Protection Rules

- ✅ **Require a pull request before merging**
  - ✅ Require approvals: 1 (adjust based on team size)
  - ✅ Dismiss stale pull request approvals when new commits are pushed
  - ❌ Require review from Code Owners (optional, configure CODEOWNERS if desired)

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging

- ✅ **Require conversation resolution before merging**

- ✅ **Do not allow bypassing the above settings** (no admin bypass)

- ✅ **Restrict who can push to matching branches**
  - Nobody should push directly to main (all changes via PR)

### Additional Recommended Settings

- ✅ **Require signed commits** (if your team uses GPG signing)
- ✅ **Include administrators** (admins follow the same rules)
- ✅ **Allow force pushes**: ❌ Disabled
- ✅ **Allow deletions**: ❌ Disabled

---

## Option 1: GitHub Web UI (Manual Setup)

### Step 1: Navigate to Branch Protection Settings

1. Go to your repository on GitHub
2. Click **Settings** (repository settings, not user settings)
3. In the left sidebar, click **Branches**
4. Under "Branch protection rules", click **Add branch protection rule**

### Step 2: Target the Main Branch

1. In "Branch name pattern", enter: `main`

### Step 3: Configure Protection Rules

#### Require Pull Requests

- ✅ Check **Require a pull request before merging**
- Set **Required number of approvals before merging**: `1`
- ✅ Check **Dismiss stale pull request approvals when new commits are pushed**

#### Require Status Checks

- ✅ Check **Require status checks to pass before merging**
- ✅ Check **Require branches to be up to date before merging**
- In the search box, type and add these status checks:
  - `tier1-selftest`
  - `ci-security`
  - `ci-docs`
  - `ci-policy-verify`

  **Note:** Status checks only appear after they've run at least once. If you don't see them, merge one PR first, then come back to add them.

#### Additional Settings

- ✅ Check **Require conversation resolution before merging**
- ✅ Check **Do not allow bypassing the above settings**
- ✅ Check **Restrict who can push to matching branches**
  - Leave the list empty (nobody can push directly)

#### Force Push and Deletion

- ❌ Uncheck **Allow force pushes**
- ❌ Uncheck **Allow deletions**

### Step 4: Save

Click **Create** at the bottom of the page.

### Step 5: Verify

Try to push directly to main:

```bash
git checkout main
echo "test" >> README.md
git commit -am "Test: should be blocked"
git push origin main
```

You should see an error like:

```
! [remote rejected] main -> main (protected branch hook declined)
```

This means branch protection is working correctly.

---

## Option 2: GitHub CLI (Automated Setup)

If you prefer automation or need to configure multiple repositories, use the provided script.

### Quick Start

```bash
# From repository root
.github/scripts/setup-branch-protection.sh
```

The script will:
1. Detect the repository owner and name from `git remote`
2. Configure branch protection rules via GitHub API
3. Enable required status checks
4. Block direct pushes to main

### What the Script Does

The script uses `gh api` to configure:

- Pull request requirements (1 approval, dismiss stale reviews)
- Required status checks (tier1-selftest, ci-security, ci-docs, ci-policy-verify)
- Branch restrictions (no direct pushes, no force-push, no deletion)
- Enforcement for administrators

### Prerequisites

1. **Install `gh` CLI:**

   ```bash
   # On Ubuntu/Debian
   sudo apt install gh

   # On macOS
   brew install gh

   # On other systems
   # See: https://github.com/cli/cli#installation
   ```

2. **Authenticate with GitHub:**

   ```bash
   gh auth login
   ```

   Follow the prompts to authenticate. Choose:
   - GitHub.com (not GitHub Enterprise)
   - HTTPS or SSH (your preference)
   - Authenticate via browser

3. **Verify permissions:**

   ```bash
   gh auth status
   ```

   Ensure your token has `repo` scope.

### Run the Script

```bash
# From repository root
.github/scripts/setup-branch-protection.sh
```

### Customize the Script

If you want different settings, edit `.github/scripts/setup-branch-protection.sh`:

```bash
# Change required approvals
"required_approving_review_count": 2,  # Default: 1

# Add more required status checks
"contexts": [
  "tier1-selftest",
  "ci-security",
  "ci-docs",
  "ci-policy-verify",
  "my-custom-check"  # Add your own
],

# Allow admins to bypass (not recommended)
"enforce_admins": false  # Default: true
```

### Troubleshooting

**Error: "Resource not accessible by integration"**

- Your GitHub token needs `repo` scope
- Re-authenticate: `gh auth login` and grant full repo access

**Error: "Branch not found"**

- The `main` branch must exist before configuring protection
- Push at least one commit to `main` first

**Error: "Required status check 'tier1-selftest' not found"**

- Status checks must run at least once before they can be required
- Merge one PR first, then re-run the script

---

## Option 3: Manual gh API Commands

If you want fine-grained control, use `gh api` directly:

### Enable Branch Protection

```bash
gh api \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  /repos/OWNER/REPO/branches/main/protection \
  -f required_pull_request_reviews[required_approving_review_count]=1 \
  -f required_pull_request_reviews[dismiss_stale_reviews]=true \
  -f required_status_checks[strict]=true \
  -F required_status_checks[contexts][]=tier1-selftest \
  -F required_status_checks[contexts][]=ci-security \
  -F required_status_checks[contexts][]=ci-docs \
  -F required_status_checks[contexts][]=ci-policy-verify \
  -f restrictions=null \
  -f enforce_admins=true \
  -f allow_force_pushes[enabled]=false \
  -f allow_deletions[enabled]=false \
  -f required_conversation_resolution[enabled]=true
```

Replace `OWNER/REPO` with your repository (e.g., `EffortlessMetrics/Rust-Template`).

### View Current Protection

```bash
gh api /repos/OWNER/REPO/branches/main/protection
```

### Remove Protection (Emergency Only)

```bash
gh api --method DELETE /repos/OWNER/REPO/branches/main/protection
```

---

## Verification

After configuring branch protection, verify it's working:

### Test 1: Direct Push Blocked

```bash
git checkout main
git pull origin main
echo "# Test" >> test.md
git add test.md
git commit -m "Test: should be blocked"
git push origin main
```

**Expected:** Push rejected with "protected branch hook declined"

### Test 2: PR Workflow Works

```bash
git checkout -b test-branch-protection
echo "# Test PR" >> test-pr.md
git add test-pr.md
git commit -m "Test: PR workflow"
git push origin test-branch-protection
gh pr create --title "Test: Branch Protection" --body "Testing PR workflow"
```

**Expected:**
- PR created successfully
- Status checks run automatically
- Merge button disabled until checks pass and approval received

### Test 3: Check Protection Settings

```bash
gh api /repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/branches/main/protection | jq
```

**Expected:** JSON output showing your configured rules

---

## Recommended Status Checks

These CI workflows should be required before merging:

| Status Check | Purpose | Workflow File |
|--------------|---------|---------------|
| **tier1-selftest** | Full governance validation (all 8 gates) | `.github/workflows/tier1-selftest.yml` |
| **ci-security** | Cargo audit, deny, and supply chain checks | `.github/workflows/ci-security.yml` |
| **ci-docs** | Documentation consistency (version alignment) | `.github/workflows/ci-docs.yml` |
| **ci-policy-verify** | OPA policy tests for config compliance | `.github/workflows/ci-policy-verify.yml` |

Optional but recommended:

| Status Check | Purpose | Workflow File |
|--------------|---------|---------------|
| **ci-coverage** | Code coverage thresholds | `.github/workflows/ci-coverage.yml` |
| **ci-lints** | Extended linting (clippy pedantic) | `.github/workflows/ci-lints.yml` |
| **ci-msrv** | Minimum supported Rust version | `.github/workflows/ci-msrv.yml` |

### Adding New Required Checks

To require additional checks:

**Via UI:**
1. Go to Settings → Branches → Edit rule for `main`
2. Search for the status check name in the search box
3. Click to add it to required checks
4. Click Save changes

**Via gh CLI:**

```bash
# Get current protection config
gh api /repos/OWNER/REPO/branches/main/protection > protection.json

# Edit protection.json to add your check to required_status_checks.contexts

# Update protection
gh api --method PUT /repos/OWNER/REPO/branches/main/protection \
  --input protection.json
```

---

## Team-Specific Customizations

### Small Teams (1-3 developers)

- Required approvals: `1`
- Status checks: `tier1-selftest` only (for speed)
- Allow self-merge after approval

### Medium Teams (4-10 developers)

- Required approvals: `1-2`
- Status checks: `tier1-selftest`, `ci-security`, `ci-docs`
- Require conversation resolution
- Dismiss stale reviews

### Large Teams (10+ developers)

- Required approvals: `2+`
- Status checks: All recommended + optional checks
- Code owners review required (configure `.github/CODEOWNERS`)
- Restrict push access to specific teams

---

## Emergency Bypass Procedure

If you need to bypass branch protection in an emergency:

### Option 1: Temporary Disable (Admin Only)

1. Go to Settings → Branches
2. Edit the `main` protection rule
3. Uncheck **Do not allow bypassing the above settings**
4. Make your emergency change
5. **Immediately re-enable** the bypass prevention

### Option 2: Use a PR with Admin Merge

1. Create a PR as normal
2. Have admin review and approve
3. Admin can merge even with failing checks (if bypass is enabled for admins)
4. Create follow-up issue to fix the failing checks

**Note:** Emergency bypasses should be rare and documented. If you're bypassing often, your checks are too strict or your process needs adjustment.

---

## Common Issues

### Issue: "Status check not found"

**Cause:** The status check hasn't run yet, so GitHub doesn't know about it.

**Solution:**
1. Merge one PR without requiring that check
2. Let the CI workflow run
3. Then add it as a required check

### Issue: "Can't merge even with checks passing"

**Cause:** Branch is out of date or conversations not resolved.

**Solution:**
- Click "Update branch" to merge latest main
- Resolve all PR conversations
- Ensure all required reviews are approved

### Issue: "Admin can't push directly"

**Cause:** "Include administrators" is checked (this is good!).

**Solution:**
- This is working as intended
- Admins should use PRs like everyone else
- Governance applies equally to all roles

### Issue: "Script fails with 403 Forbidden"

**Cause:** GitHub token lacks permissions.

**Solution:**

```bash
gh auth refresh -s repo
```

Grant the `repo` scope when prompted.

---

## Next Steps

After configuring branch protection:

1. **Update team documentation** - Let your team know about the new workflow
2. **Test the workflow** - Create a test PR and verify checks run correctly
3. **Monitor initial PRs** - Watch for issues with required checks
4. **Adjust as needed** - Add/remove required checks based on team velocity

---

## Related Documentation

- `docs/explanation/why-this-exists.md` - Why governance matters
- `docs/ROADMAP.md` - Roadmap tracking enforcement gaps
- `.github/workflows/tier1-selftest.yml` - The main governance gate
- `CLAUDE.md` - Agent guidance on working within governance

---

## Summary

Branch protection is the final enforcement layer for the Rust-as-Spec template. It ensures:

✅ All changes pass selftest before merging
✅ Peer review happens before changes reach main
✅ History is protected from force-pushes
✅ Governance contracts are enforced at the platform level

**Setup time:** 10 minutes
**Maintenance:** None (automated via GitHub)
**Value:** Prevents entire classes of governance bypass
