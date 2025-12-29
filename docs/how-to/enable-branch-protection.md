---
id: GUIDE-TPL-ENABLE-PROTECTION-001
title: Enable Branch Protection When CI Returns
doc_type: how_to
status: published
audience: maintainers, platform-engineers
tags: [security, github, branch-protection, ci, enforcement, governance]
stories: [US-TPL-PLT-001]
requirements: [REQ-PLT-RELEASE-SAFETY, REQ-PLT-SECURITY-GOVERNANCE]
acs: [AC-PLT-011, AC-PLT-012, AC-PLT-013]
adrs: [ADR-0005, ADR-0006, ADR-0017]
last_updated: 2025-12-27
---

# How-to: Enable Branch Protection When CI Returns

**Time:** 15-30 minutes
**Prerequisites:** Repository admin access, `gh` CLI installed and authenticated

This checklist document queues up the enforcement steps to apply when CI is back online.
Follow these steps in order to fully secure the `main` branch.

---

## Pre-Flight Checklist

Before enabling branch protection, verify CI is operational:

```bash
# 1. Verify CI workflows exist and are valid
ls -la .github/workflows/tier1-selftest.yml
ls -la .github/workflows/ci-security.yml
ls -la .github/workflows/ci-docs.yml
ls -la .github/workflows/ci-policy-verify.yml

# 2. Verify gh CLI is authenticated
gh auth status

# 3. Check repository access
gh repo view --json nameWithOwner -q .nameWithOwner
```

---

## Phase 1: Trigger CI Workflows (Required First)

GitHub cannot require status checks until they have run at least once. Create a test PR to trigger all workflows:

```bash
# 1. Create a test branch
git checkout main
git pull origin main
git checkout -b test/ci-warmup

# 2. Make a trivial change that triggers all CI workflows
echo "# CI Warmup $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> .github/CI_WARMUP.md

# 3. Commit and push
git add .github/CI_WARMUP.md
git commit -m "chore: CI warmup to register status checks"
git push origin test/ci-warmup

# 4. Create PR
gh pr create --title "chore: CI warmup" --body "Triggering CI workflows to register status checks for branch protection."

# 5. Wait for CI to complete (check Actions tab)
gh pr checks test/ci-warmup --watch
```

After CI completes:
- [ ] `tier1-selftest` workflow ran successfully
- [ ] `ci-security` workflow ran (or was skipped for expected reasons)
- [ ] `ci-docs` workflow ran (if docs paths changed)
- [ ] `ci-policy-verify` workflow ran (if policy paths changed)

**Before closing the PR, discover the exact check names (this is critical!):**

```bash
# Get the PR number
PR_NUM=$(gh pr view test/ci-warmup --json number -q .number)

# List ALL check runs and their exact names
gh api "/repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/commits/$(git rev-parse HEAD)/check-runs" \
  --jq '.check_runs[] | {name: .name, status: .status, conclusion: .conclusion}' | jq -s '.'

# Or use the simpler PR checks view
gh pr checks test/ci-warmup
```

**Record the exact check names you see.** They are typically the **job name** from the workflow, not the workflow file name. For example:
- Workflow file: `tier1-selftest.yml` → Check name might be: `selftest` or `tier1-selftest / build`
- Workflow file: `ci-security.yml` → Check name might be: `security-scan` or `audit`

Common patterns:
- If workflow has `jobs: { build: ... }` → check name is `build`
- If workflow has `jobs: { selftest: ... }` → check name is `selftest`
- Matrix jobs appear as `job-name (matrix-value)`

**Save these names** - you'll need them for branch protection configuration.

Clean up:

```bash
# Close PR without merging (we don't need the warmup file)
gh pr close test/ci-warmup --delete-branch
git checkout main
```

---

## Phase 2: Enable Branch Protection

> **⚠️ Critical:** Use the exact check names you discovered in Phase 1, NOT the workflow file names.
> Using incorrect names causes "Expected — waiting for status to be reported" errors that block all PRs.

### Option A: Use the Setup Script (After Discovering Check Names)

```bash
# First, edit the script to use YOUR discovered check names
# Look for the REQUIRED_CHECKS variable and update it
nano .github/scripts/setup-branch-protection.sh

# Then run the script
.github/scripts/setup-branch-protection.sh
```

The script configures:
- Required status checks: **(must match your discovered check names)**
- Required approvals: 1
- Dismiss stale reviews: enabled
- Enforce for admins: enabled
- Block force pushes and deletions

### Option B: Manual Setup via GitHub UI

1. Go to **Settings** > **Branches** > **Add branch protection rule**
2. Set **Branch name pattern**: `main`
3. Enable these settings:

| Setting | Value |
| ------- | ----- |
| **Require a pull request before merging** | Enabled |
| Required approvals | 1 |
| Dismiss stale reviews | Enabled |
| **Require status checks to pass** | Enabled |
| Require branches to be up to date | Enabled |
| Status checks (search and add): | *(use your discovered check names from Phase 1)* |
| **Require conversation resolution** | Enabled |
| **Do not allow bypassing the above settings** | Enabled |
| **Restrict who can push** | Enabled (leave list empty) |
| **Allow force pushes** | Disabled |
| **Allow deletions** | Disabled |

1. Click **Create** or **Save changes**

### Option C: Manual gh API Command

```bash
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)

# Replace these with YOUR discovered check names from Phase 1
CHECK1="your-first-check-name"
CHECK2="your-second-check-name"
# Add more as needed...

gh api \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/$REPO/branches/main/protection" \
  -f required_status_checks="{\"strict\":true,\"contexts\":[\"$CHECK1\",\"$CHECK2\"]}" \
  -f enforce_admins=true \
  -f required_pull_request_reviews='{"dismiss_stale_reviews":true,"required_approving_review_count":1}' \
  -f restrictions=null \
  -f required_conversation_resolution='{"enabled":true}' \
  -f allow_force_pushes='{"enabled":false}' \
  -f allow_deletions='{"enabled":false}'
```

---

## Phase 3: Verify Branch Protection

```bash
# 1. Verify protection is active
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
gh api "/repos/$REPO/branches/main/protection" | jq

# 2. Test that direct push is blocked
git checkout main
echo "# test" >> test.md
git add test.md
git commit -m "test: should be blocked"
git push origin main
# Expected: remote rejected (protected branch hook declined)

# 3. Clean up test commit
git reset --hard HEAD~1
```

Verification checklist:
- [ ] Direct push to main is blocked
- [ ] PR merge button shows "Checks required"
- [ ] `tier1-selftest` appears as required check
- [ ] Admin bypass is disabled

---

## Phase 4: Configure Tag Signing

For release authenticity, enable GPG-signed tags. See `docs/how-to/setup-tag-signing.md` for full instructions.

### Quick Setup

```bash
# 1. Check for existing GPG key
gpg --list-secret-keys --keyid-format LONG

# 2. If no key exists, generate one
gpg --full-generate-key
# Choose: RSA and RSA, 4096 bits, 2 years expiration

# 3. Get your key ID (the part after rsa4096/)
gpg --list-secret-keys --keyid-format LONG
# Example output: sec   rsa4096/ABCD1234EFGH5678 2024-01-15

# 4. Configure Git to use the key
git config --global user.signingkey ABCD1234EFGH5678
git config --global tag.gpgSign true

# 5. Export public key for GitHub
gpg --armor --export ABCD1234EFGH5678
# Copy output and add at: https://github.com/settings/keys

# 6. Test signing
git tag -s test-signing -m "Test GPG signing"
git tag -v test-signing
# Expected: "Good signature from..."
git tag -d test-signing
```

Tag signing checklist:
- [ ] GPG key generated (or existing key identified)
- [ ] Git configured to sign tags by default
- [ ] Public key added to GitHub account
- [ ] Test tag created and verified locally

---

## Phase 5: Enable Additional CI Governance (Optional)

These additional workflows can be required based on your team's needs:

### Recommended for Most Teams

| Workflow      | File              | When to Require                        |
| ------------- | ----------------- | -------------------------------------- |
| `ci-coverage` | `ci-coverage.yml` | When coverage thresholds are important |
| `ci-lints` | `ci-lints.yml` | For stricter clippy enforcement |
| `ci-skills` | `ci-skills.yml` | When modifying Skills definitions |
| `ci-agents` | `ci-agents.yml` | When modifying Agent definitions |

### Add Additional Required Checks

```bash
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)

# Get current protection
gh api "/repos/$REPO/branches/main/protection" > protection.json

# Edit protection.json to add more contexts to required_status_checks.contexts
# Then update:
gh api --method PUT "/repos/$REPO/branches/main/protection" --input protection.json
```

---

## Phase 6: Document Enforcement in Team Practices

### Update CONTRIBUTING.md

Add or verify this section exists:

```markdown
## Branch Protection

The `main` branch is protected. All changes require:

1. **Pull request** with at least 1 approval
2. **Passing status checks** (see required checks in Settings > Branches)
3. **Resolved conversations**
4. **No direct pushes** (even for admins)

### Release Tags

All release tags MUST be GPG-signed. See `docs/how-to/setup-tag-signing.md`.
```

### Notify Team

```bash
# Create an issue to track the change
gh issue create \
  --title "Branch protection enabled on main" \
  --body "Branch protection has been enabled on the \`main\` branch.

## What Changed
- Direct pushes to main are now blocked
- PRs require at least 1 approval
- Required status checks are now enforced (see Settings > Branches)
- Force pushes and branch deletion are disabled

## Action Required
- All changes must go through PRs
- Ensure your local branch is up to date before pushing
- If you encounter issues, see docs/how-to/setup-branch-protection.md

## For Release Engineers
- Release tags must be GPG-signed
- See docs/how-to/setup-tag-signing.md for setup instructions
"
```

---

## Troubleshooting

### "Status check not found" when adding required checks

**Cause:** The workflow has never run, so GitHub doesn't know about it.

**Solution:** Create a PR that triggers the workflow, wait for it to complete, then add the check.

### "Expected — waiting for status to be reported" (PR blocked forever)

**Cause:** The required check name doesn't match any actual check run. This is the most common mistake - using workflow file names instead of job names.

**Solution:**
1. Go to any recent PR and look at the Checks tab
2. Note the exact check names (they're job names, not file names)
3. Update branch protection to use those exact names

```bash
# Discover actual check names from a recent commit
gh api "/repos/$(gh repo view --json nameWithOwner -q .nameWithOwner)/commits/$(git rev-parse main)/check-runs" \
  --jq '.check_runs[].name' | sort -u
```

Then update branch protection with the correct names.

### "Resource not accessible by integration"

**Cause:** GitHub token lacks `repo` scope.

**Solution:**

```bash
gh auth refresh -s repo
```

### Branch protection script fails

**Cause:** Various permission or configuration issues.

**Solution:** Check the detailed error output and refer to `.github/scripts/setup-branch-protection.sh` for common issues.

### Admin can't merge with failing checks

**Cause:** "Do not allow bypassing" is enabled (this is correct!).

**Solution:** Fix the failing checks. Admins should follow the same rules as everyone else.

---

## Verification Summary Checklist

Use this checklist to verify all enforcement is in place:

### Branch Protection

- [ ] Direct push to `main` blocked
- [ ] PR required before merging
- [ ] At least 1 approval required
- [ ] Stale reviews dismissed on new commits
- [ ] Required status checks configured (using exact names from Phase 1)
- [ ] Branches must be up to date before merging
- [ ] Conversation resolution required
- [ ] Admin bypass disabled
- [ ] Force pushes disabled
- [ ] Branch deletion disabled

### Tag Signing

- [ ] GPG key configured in Git
- [ ] Public key added to GitHub
- [ ] `tag.gpgSign = true` in Git config
- [ ] Test tag verified locally

### Team Communication

- [ ] CONTRIBUTING.md updated
- [ ] Team notified via issue or announcement

---

## Related Documentation

- `docs/how-to/setup-branch-protection.md` - Detailed branch protection setup
- `docs/how-to/setup-tag-signing.md` - GPG tag signing configuration
- `.github/scripts/setup-branch-protection.sh` - Automation script
- `.github/workflows/tier1-selftest.yml` - Primary governance gate
- `docs/ROADMAP.md` - Overall project status and enforcement gaps

---

## Summary

After following this guide, your repository will have:

1. **Branch protection** preventing direct pushes to main
2. **Required CI checks** enforcing governance before merge
3. **GPG-signed tags** for release authenticity
4. **Team documentation** ensuring everyone understands the new workflow

This establishes the enforcement layer that makes the Rust-as-Spec governance model work at the platform level.
