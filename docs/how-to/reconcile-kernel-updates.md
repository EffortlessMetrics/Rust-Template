<!-- doclint:disable orphan-version -->
---
id: GUIDE-TPL-RECONCILE-001
title: Reconcile Kernel Updates in Your Fork
doc_type: how-to
status: published
audience: maintainers, platform-engineers, fork-owners
tags: [fork, maintenance, kernel, upstream, sync]
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-OVERRIDE-PATH, REQ-TPL-EXAMPLE-FORK]
acs: [AC-TPL-OVERRIDE-DOC, AC-TPL-OVERRIDE-TRACEABLE]
adrs: [ADR-0005]
last_updated: 2025-12-02
---

# Reconcile Kernel Updates in Your Fork

**Audience:** Fork maintainers who need to pull in upstream kernel improvements
**Time:** 30-90 minutes (depends on conflicts)
**Prerequisites:** Active fork with customizations, understanding of git merge strategies

---

## The Problem

Your fork customized the template months ago. The upstream kernel has since:

- Added new ACs for better governance
- Fixed critical bugs
- Improved platform APIs
- Updated dependencies

**You want:** Upstream improvements **without** losing your fork's customizations.

**This guide shows:** How to safely merge kernel updates while preserving fork-specific changes.

---

## Before You Start

### 1. Understand What You'll Merge

Kernel updates typically include:

- **New or changed ACs** in `specs/spec_ledger.yaml`
- **BDD scenario updates** in `specs/features/*.feature`
- **Bug fixes** in `crates/` code
- **Dependency bumps** in `Cargo.toml`
- **Documentation improvements** in `docs/`

**Key insight:** Not all kernel changes will conflict with your fork. Most merges are clean.

### 2. Check Your Fork's Health First

Before pulling upstream, ensure your fork is in a known good state:

```bash
# Verify your fork is healthy
cargo xtask selftest

# Capture current AC status
cargo xtask ac-status > /tmp/fork-before-merge.txt

# Commit any uncommitted changes
git status
git add .
git commit -m "chore: pre-merge checkpoint"
```

**Why:** You need a clean baseline to distinguish merge issues from pre-existing problems.

---

## The Reconciliation Process

### Step 1: Set Up Upstream Remote

If you haven't already, add the template repo as an upstream remote:

```bash
# Add upstream remote
git remote add upstream https://github.com/EffortlessMetrics/Rust-Template.git

# Verify remotes
git remote -v
# Should show:
#   origin     git@github.com:your-org/your-fork.git (fetch/push)
#   upstream   https://github.com/EffortlessMetrics/Rust-Template.git (fetch/push)
```

**One-time setup.** You won't need to repeat this.

### Step 2: Fetch Upstream Changes

```bash
# Fetch latest kernel changes
git fetch upstream main

# View what's new
git log HEAD..upstream/main --oneline --graph --decorate

# See files that changed
git diff HEAD..upstream/main --name-status
```

**Expected output:**

```
* a1b2c3d (upstream/main) feat: add AC-PLT-XYZ for better traceability
* d4e5f6g fix: resolve policy test false positives
* h7i8j9k deps: bump axum to 0.7.9
```

**Tip:** Focus on commits since your last merge. Use `git log --since="2025-11-01"` to filter.

### Step 3: Create a Merge Branch

```bash
# Create a branch for the merge
git checkout -b merge/kernel-updates-2025-12-02

# Merge upstream main
git merge upstream/main
```

**Three possible outcomes:**

#### Outcome A: Clean Merge (Best Case)

```
Auto-merging crates/app-http/src/main.rs
Merge made by the 'ort' strategy.
 6 files changed, 142 insertions(+), 23 deletions(-)
```

**Action:** Skip to Step 4 (Validate).

#### Outcome B: Conflicts in Non-Critical Files

```
Auto-merging README.md
CONFLICT (content): Merge conflict in README.md
Auto-merging CHANGELOG.md
CONFLICT (content): Merge conflict in CHANGELOG.md
```

**Action:** Resolve conflicts (see Step 3a), then proceed to Step 4.

#### Outcome C: Conflicts in Specs or Code

```
Auto-merging specs/spec_ledger.yaml
CONFLICT (content): Merge conflict in specs/spec_ledger.yaml
Auto-merging crates/xtask/src/commands/doctor.rs
CONFLICT (content): Merge conflict in crates/xtask/src/commands/doctor.rs
```

**Action:** Carefully resolve conflicts (see Step 3b), then proceed to Step 4.

### Step 3a: Resolve Documentation Conflicts

**Common conflict locations:**

- `README.md` - Service name, description
- `CHANGELOG.md` - Version history
- `CLAUDE.md` - Service metadata

**Strategy:** Keep your fork's customizations, but incorporate upstream structural improvements.

**Example conflict in README.md:**

```markdown
<<<<<<< HEAD
# My Service

My custom service description.
=======
# Rust-as-Spec Template

A governed service template.
>>>>>>> upstream/main
```

**Resolution:**

```markdown
# My Service

My custom service description.

Based on the Rust-as-Spec template (v3.3.6).
```

**Commands:**

```bash
# Edit conflicted file
vim README.md

# Mark as resolved
git add README.md

# Continue merge
git merge --continue
```

### Step 3b: Resolve Spec Ledger Conflicts

**Most critical conflict:** `specs/spec_ledger.yaml`

**Conflict scenario:** Upstream added a new AC. You customized the same REQ.

**Example conflict:**

```yaml
<<<<<<< HEAD
# Your fork's version
- id: REQ-PLT-ONBOARDING
  title: "New developer onboarding is guided and validated"
  tags: [platform, devex]
  must_have_ac: true
  note: "Nix is optional in our fork"  # Your customization
  acceptance_criteria:
    - id: AC-PLT-001
      text: "`cargo xtask doctor` validates Rust and git, optionally checks Nix"
      tags: [kernel]
      must_have_ac: true
=======
# Upstream's version
- id: REQ-PLT-ONBOARDING
  title: "New developer onboarding is guided and validated"
  tags: [platform, devex]
  must_have_ac: true
  acceptance_criteria:
    - id: AC-PLT-001
      text: "`cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance"
      tags: [kernel]
      must_have_ac: true
    - id: AC-PLT-NEW-002  # ← NEW AC from upstream
      text: "`cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps"
      tags: [kernel]
      must_have_ac: true
>>>>>>> upstream/main
```

**Resolution strategy:**

1. **Keep your AC text customizations** (e.g., "optionally checks Nix")
2. **Incorporate new ACs** from upstream (e.g., AC-PLT-NEW-002)
3. **Preserve your fork's notes**

**Resolved version:**

```yaml
- id: REQ-PLT-ONBOARDING
  title: "New developer onboarding is guided and validated"
  tags: [platform, devex]
  must_have_ac: true
  note: "Nix is optional in our fork"  # Preserved fork customization
  acceptance_criteria:
    - id: AC-PLT-001
      text: "`cargo xtask doctor` validates Rust and git, optionally checks Nix"
      tags: [kernel]
      must_have_ac: true
      note: "Fork customization: Nix is optional"
    - id: AC-PLT-NEW-002  # New AC from upstream
      text: "`cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps"
      tags: [kernel]
      must_have_ac: true
      tests:
        - { type: integration, tag: "@AC-PLT-NEW-002", file: "specs/features/xtask_devex.feature" }
```

**Commands:**

```bash
# Edit spec_ledger.yaml carefully
vim specs/spec_ledger.yaml

# Validate YAML syntax
cargo xtask ac-status

# If YAML is valid, mark as resolved
git add specs/spec_ledger.yaml
```

### Step 3c: Resolve Code Conflicts

**Common conflict:** Upstream changed a function you also modified.

**Example conflict in `crates/xtask/src/commands/doctor.rs`:**

```rust
<<<<<<< HEAD
// Your fork's version
fn check_nix() -> Result<String> {
    // Optional check - warn if missing
    match which::which("nix-shell") {
        Ok(_) => Ok("✓ Nix installed".to_string()),
        Err(_) => Ok("⚠️  Nix not found (optional)".to_string()),
    }
}
=======
// Upstream's version
fn check_nix() -> Result<String> {
    // Required check - fail if missing
    which::which("nix-shell")
        .map(|_| "✓ Nix installed".to_string())
        .context("Nix is required for this template")
}
>>>>>>> upstream/main
```

**Resolution:**

Keep your fork's behavior (Nix optional), but incorporate any bug fixes or improvements from upstream:

```rust
fn check_nix() -> Result<String> {
    // Fork customization: Nix is optional
    match which::which("nix-shell") {
        Ok(_) => Ok("✓ Nix installed".to_string()),
        Err(_) => {
            eprintln!("⚠️  Nix not found (optional in this fork)");
            Ok("⚠️  Nix not found (optional)".to_string())
        }
    }
}
```

**Commands:**

```bash
# Edit the file
vim crates/xtask/src/commands/doctor.rs

# Test your changes
cargo xtask check

# Mark as resolved
git add crates/xtask/src/commands/doctor.rs
```

### Step 4: Validate the Merge

After resolving conflicts, validate your merged state:

```bash
# 1. Check compilation
cargo xtask check

# 2. Run changed tests
cargo xtask test-changed

# 3. Check AC status (compare to pre-merge)
cargo xtask ac-status > /tmp/fork-after-merge.txt
diff /tmp/fork-before-merge.txt /tmp/fork-after-merge.txt

# 4. Run full selftest (critical)
cargo xtask selftest
```

**Expected:** Selftest should be **green** after the merge.

**If selftest fails:**

1. **New AC failures:** The upstream added ACs that aren't passing in your fork yet.
   - **Option A:** Mark new ACs as `must_have_ac: false` temporarily (see "Step 5: Handle New ACs")
   - **Option B:** Implement the new ACs before merging (longer, but cleaner)

2. **Broken existing ACs:** The merge introduced regressions.
   - **Action:** Review conflicts again. You may have accidentally deleted fork-specific code.

3. **Test failures unrelated to ACs:** Code bugs introduced by merge.
   - **Action:** Use `cargo xtask test-changed` to isolate failing tests, fix them, re-run selftest.

### Step 5: Handle New ACs from Upstream

**Scenario:** Upstream added `AC-PLT-NEW-002`. Your fork doesn't implement it yet.

**Options:**

#### Option A: Mark as Optional (Recommended for Short-Term)

```yaml
# In your fork's specs/spec_ledger.yaml
- id: AC-PLT-NEW-002
  text: "`cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps"
  tags: [template]  # Changed from [kernel]
  must_have_ac: false  # Mark as optional in fork
  note: "Upstream addition - not yet implemented in fork. Tracked in TASK-FORK-123."
  tests:
    - { type: integration, tag: "@AC-PLT-NEW-002", file: "specs/features/xtask_devex.feature" }
```

**Then:**

```bash
# Create task to implement it later
echo "- id: TASK-FORK-123
  title: Implement AC-PLT-NEW-002 (dev-up command)
  status: Todo
  requirement: REQ-PLT-ONBOARDING
  acs: [AC-PLT-NEW-002]
  owner: team
" >> specs/tasks.yaml

# Verify selftest passes with AC marked optional
cargo xtask selftest
```

#### Option B: Implement Immediately (Recommended for Long-Term)

```bash
# 1. Read the AC requirements
# (Check specs/spec_ledger.yaml and linked tests)

# 2. Implement the feature
# (Edit crates/xtask/src/commands/*.rs)

# 3. Wire BDD scenarios
# (Edit specs/features/xtask_devex.feature)

# 4. Test the AC
cargo xtask test-ac AC-PLT-NEW-002

# 5. Verify selftest passes
cargo xtask selftest
```

#### Option C: Skip Entirely (Use Sparingly)

If the new AC is genuinely not applicable to your fork:

```yaml
# In your fork's specs/spec_ledger.yaml
- id: AC-PLT-NEW-002
  text: "`cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps"
  tags: [template, skipped]
  must_have_ac: false
  note: "Skipped in fork - we use a custom onboarding script instead."
  tests: []  # No tests - AC not implemented
```

**Document why:**

```bash
cat > docs/adr/ADR-FORK-005.md <<EOF
---
id: ADR-FORK-005
title: Skip AC-PLT-NEW-002 (dev-up command)
status: accepted
date: 2025-12-02
---

## Context
Upstream added AC-PLT-NEW-002 requiring a \`dev-up\` command.
Our fork uses a custom onboarding script (\`scripts/onboard.sh\`).

## Decision
Skip AC-PLT-NEW-002 in our fork. Mark as \`must_have_ac: false\`.

## Consequences
- No \`cargo xtask dev-up\` in our fork
- Developers use \`scripts/onboard.sh\` instead
- Must document this difference in fork README
EOF
```

### Step 6: Update BDD Scenarios

If upstream changed feature files, you may need to reconcile them:

```bash
# Check for conflicts in feature files
git status | grep "specs/features"

# If conflicts exist, resolve them
vim specs/features/xtask_devex.feature

# Validate Gherkin syntax
cargo xtask bdd --dry-run

# Run BDD tests
cargo xtask bdd
```

**Common scenario:** Upstream added a new scenario for a new AC.

**Example:**

```gherkin
# Upstream added this scenario
@AC-PLT-NEW-002
Scenario: dev-up runs doctor, hooks, and check
  When I run "cargo xtask dev-up"
  Then the command succeeds
  And stdout contains "Installing git hooks"
  And stdout contains "Running doctor"
  And stdout contains "Running check"
```

**Your fork's choice:**

- **Keep it:** If you implemented the AC
- **Skip it:** If you marked the AC as optional

```gherkin
# Fork version if skipping
@AC-PLT-NEW-002 @skip
Scenario: dev-up runs doctor, hooks, and check
  # Skipped in fork - we use scripts/onboard.sh instead
```

### Step 7: Commit the Merge

```bash
# Stage all resolved files
git add .

# Commit with detailed message
git commit -m "$(cat <<'EOF'
merge: reconcile kernel updates from upstream/main

Merged upstream kernel changes from v3.3.6 -> v3.4.0.

Changes incorporated:
- AC-PLT-NEW-002: dev-up command (marked optional in fork)
- Bug fix in policy test harness
- Dependencies updated (axum 0.7.9, serde 1.0.215)

Fork customizations preserved:
- AC-PLT-001: Nix remains optional (fork policy)
- Custom error handling in doctor.rs

Conflicts resolved in:
- specs/spec_ledger.yaml (preserved fork note about Nix)
- crates/xtask/src/commands/doctor.rs (kept fork behavior)
- README.md (merged service description)

Validation:
- cargo xtask selftest: PASS
- cargo xtask ac-status: All kernel ACs passing

References:
- Upstream PR: EffortlessMetrics/Rust-Template#123
- Fork ADR: ADR-FORK-005 (skip AC-PLT-NEW-002)
EOF
)"
```

### Step 8: Test in CI

```bash
# Push merge branch
git push origin merge/kernel-updates-2025-12-02

# Create PR
gh pr create \
  --title "Reconcile kernel updates from v3.3.6 -> v3.4.0" \
  --body "$(cat <<'EOF'
## Summary
Merges upstream kernel improvements while preserving fork customizations.

## Upstream Changes
- AC-PLT-NEW-002: dev-up command (marked optional in fork)
- Policy test bug fix
- Dependency updates (axum, serde)

## Fork Customizations Preserved
- AC-PLT-001: Nix optional (fork policy)
- Custom error handling in doctor.rs

## Validation
- [x] `cargo xtask selftest` passes locally
- [x] No regressions in fork-specific ACs
- [x] Conflicts resolved in spec_ledger.yaml, doctor.rs, README.md
- [ ] CI tier1-selftest passes

## Follow-up
- [ ] Implement AC-PLT-NEW-002 (TASK-FORK-123)
- [ ] Update fork README with kernel version bump
EOF
)"

# Watch CI
gh pr checks --watch
```

**Expected:** CI tier1-selftest passes.

**If CI fails but local passes:** Environment mismatch. Run `nix develop --command cargo xtask selftest` to reproduce CI exactly.

---

## Handling Specific Conflict Scenarios

### Scenario 1: Upstream Removed an AC You Customized

**Problem:** Upstream removed `AC-PLT-OLD-001`. You customized it in your fork.

**Solution:**

```yaml
# In your fork's specs/spec_ledger.yaml

# Option A: Keep the AC as fork-specific
- id: AC-PLT-OLD-001
  text: "Your customized version of the removed AC"
  tags: [fork-specific]  # New tag to indicate this is not from kernel
  must_have_ac: true  # Keep enforced in fork
  note: "Upstream removed this AC, but we still require it. See ADR-FORK-006."
  tests:
    - { type: integration, tag: "@AC-PLT-OLD-001", file: "specs/features/fork_custom.feature" }

# Option B: Remove it and migrate to upstream's replacement
# (Delete the AC, update code to match upstream's new approach)
```

**Document:**

```bash
cat > docs/adr/ADR-FORK-006.md <<EOF
---
id: ADR-FORK-006
title: Preserve AC-PLT-OLD-001 Despite Upstream Removal
status: accepted
date: 2025-12-02
---

## Context
Upstream removed AC-PLT-OLD-001 in favor of a new approach.
Our fork relies on this AC for compliance reasons.

## Decision
Keep AC-PLT-OLD-001 as fork-specific. Tag with [fork-specific].

## Consequences
- We maintain this AC independently
- Must manually reconcile future upstream changes
- Compliance requirement satisfied
EOF
```

### Scenario 2: Upstream Changed an AC You Demoted

**Problem:** Upstream changed `AC-PLT-001` text. You demoted it to optional.

**Solution:**

```yaml
# Merge conflict in specs/spec_ledger.yaml
<<<<<<< HEAD
# Your fork's version (demoted)
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust and git, optionally checks Nix"
  tags: [template]  # Demoted from [kernel]
  must_have_ac: false  # Demoted
  note: "Nix is optional in our fork"
=======
# Upstream's version (updated text)
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust, Nix, conftest, git and provides actionable diagnostics"
  tags: [kernel]
  must_have_ac: true
>>>>>>> upstream/main
```

**Resolution:**

```yaml
# Keep your demotion but take upstream's improvements
- id: AC-PLT-001
  text: "`cargo xtask doctor` validates Rust, git, optionally checks Nix and conftest, and provides actionable diagnostics"
  tags: [template]  # Keep demoted
  must_have_ac: false  # Keep optional
  note: "Fork customization: Nix is optional. Upstream improved diagnostics incorporated."
```

### Scenario 3: Dependency Version Conflicts

**Problem:** Upstream bumped `axum = "0.7.9"`. You're on `axum = "0.7.5"`.

**Solution:**

```bash
# Accept upstream's version (usually safe for patch/minor bumps)
# In Cargo.toml
axum = "0.7.9"  # From upstream

# Update lockfile
cargo update -p axum

# Test for regressions
cargo xtask check
cargo xtask test-changed

# If tests fail, investigate breaking changes
# (Check axum CHANGELOG, adjust code if needed)
```

**When to reject upstream's version:**

- You're pinned to an older version for stability
- Upstream's version introduces breaking changes you can't adopt yet

**If rejecting:**

```toml
# In Cargo.toml
axum = "0.7.5"  # Keep fork's version

# Add comment explaining why
# Pin to 0.7.5 for compatibility with internal middleware
# See TASK-FORK-124 to upgrade to 0.7.9
```

---

## Tips for Smooth Reconciliation

### 1. Merge Frequently

**Don't wait months.** Merge upstream changes every 2-4 weeks to minimize conflicts.

```bash
# Set a reminder
echo "0 9 1,15 * * cd ~/your-fork && git fetch upstream main" | crontab -
```

### 2. Track Upstream PRs

Watch template repo for relevant changes:

```bash
# Subscribe to releases
gh repo watch EffortlessMetrics/Rust-Template --releases

# Review upstream PRs before merging
gh pr list --repo EffortlessMetrics/Rust-Template --state merged --limit 10
```

### 3. Maintain a Fork Changelog

Document what you've pulled in:

```markdown
# FORK_CHANGELOG.md

## 2025-12-02: Merged upstream v3.4.0
- AC-PLT-NEW-002: dev-up command (marked optional)
- Policy test bug fix
- Dependencies: axum 0.7.9, serde 1.0.215
- Conflicts: spec_ledger.yaml, doctor.rs, README.md

## 2025-11-15: Merged upstream v3.3.6
- Initial fork creation
- Customized AC-PLT-001 (Nix optional)
```

### 4. Use ADRs for Merge Decisions

When you make significant merge choices (e.g., skip an AC, reject a dependency bump):

```bash
cargo xtask adr-new "Skip AC-PLT-XYZ in fork"
# Document the reasoning, alternatives, and consequences
```

### 5. Test Incrementally

Don't merge everything at once. If upstream has many changes:

```bash
# Merge one upstream PR at a time
git merge <commit-sha-of-upstream-pr-1>
cargo xtask selftest
git merge <commit-sha-of-upstream-pr-2>
cargo xtask selftest
```

This isolates failures to specific changes.

---

## When Upstream Breaks Your Fork

**Symptom:** After merging, selftest fails with multiple kernel AC failures.

**Possible causes:**

1. **Upstream made a breaking change** (rare, but happens)
2. **You resolved conflicts incorrectly**
3. **Upstream added new required ACs you haven't implemented**

**Diagnosis:**

```bash
# Compare AC status before/after merge
diff /tmp/fork-before-merge.txt /tmp/fork-after-merge.txt

# Identify new failures
cargo xtask ac-status | grep FAIL

# Check which tests are failing
cargo xtask bdd --fail-fast
```

**Recovery:**

### Option A: Revert and Re-Merge Carefully

```bash
# Abort the merge
git merge --abort

# Or revert the merge commit
git revert -m 1 HEAD

# Start over, merging one change at a time
git cherry-pick <upstream-commit-1>
cargo xtask selftest
git cherry-pick <upstream-commit-2>
cargo xtask selftest
```

### Option B: Mark New ACs as Optional

```yaml
# Temporarily demote new ACs that are failing
- id: AC-PLT-NEW-XYZ
  must_have_ac: false
  note: "Upstream addition - implementation in progress (TASK-FORK-125)"
```

### Option C: File Upstream Issue

If upstream truly broke something:

```bash
# File issue on template repo
gh issue create --repo EffortlessMetrics/Rust-Template \
  --title "v3.4.0 breaks forks with custom AC-PLT-001" \
  --body "Upstream change in <commit> breaks forks that customize AC-PLT-001.

  Reproduction:
  1. Fork with AC-PLT-001 marked optional
  2. Merge upstream v3.4.0
  3. Run selftest -> fails with X error

  Expected: Selftest should pass with fork customization preserved.
  Actual: Selftest fails with <error>."
```

---

## Summary

**Reconciling kernel updates is a managed process:**

1. **Fetch upstream:** `git fetch upstream main`
2. **Merge:** `git merge upstream/main`
3. **Resolve conflicts:** Preserve fork customizations, incorporate upstream improvements
4. **Handle new ACs:** Mark as optional (short-term) or implement (long-term)
5. **Validate:** `cargo xtask selftest` must pass
6. **Commit:** Document what changed and why
7. **CI:** Push and verify tier1-selftest passes

**Key principle:** Your fork's customizations are first-class. Don't blindly accept upstream changes—reconcile them with your fork's policies.

**Make this easy:** Merge frequently (every 2-4 weeks), maintain a fork changelog, and use ADRs to document merge decisions.

---

## Related Guides

- [docs/how-to/change-template-opinion.md](./change-template-opinion.md) - How to customize ACs in your fork
- [docs/how-to/FIRST_FORK.md](./FIRST_FORK.md) - Initial fork setup
- [docs/how-to/report-fork-feedback.md](./report-fork-feedback.md) - Sending feedback upstream
- [docs/how-to/maintain-kernel.md](./maintain-kernel.md) - Upstream kernel maintenance (for template maintainers)
- [docs/explanation/template-versioning.md](../explanation/template-versioning.md) - How kernel versioning works
