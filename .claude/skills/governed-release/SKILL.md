---
name: governed-release
description: |
  Version management and release tagging for the Rust-as-Spec platform cell. Use when cutting a new version, preparing for production deployment, or when user explicitly requests a release. Follows the release flow from devex_flows.yaml. IMPORTANT: Always ask user before tagging (high-risk operation).
allowed-tools:
- Read
- Grep
- Glob
- Edit
- Write
- Bash
---

# Governed Release

## When to Use

Use this Skill when:
- Cutting a new version
- User explicitly requests "cut a release" or "prepare version X.Y.Z"
- Preparing for production deployment
- Tagging a stable snapshot

**IMPORTANT:** Releases are high-risk operations. Always:
- ✅ Ask user before creating git tags
- ✅ Ensure selftest passes completely
- ✅ Never force-push to main/master

## Prerequisites

- All features are complete
- `cargo xtask selftest` passes (11/11 steps)
- CHANGELOG.md is up to date
- No outstanding policy violations
- Working on clean main branch
- User has reviewed and approved release

## Workflow

This Skill follows the **release** flow from `specs/devex_flows.yaml`.

### 1. Verify release readiness

Before starting the release process:

```bash
# Full governance validation
cargo xtask selftest

# Security audit
cargo xtask audit

# Docs validation
cargo xtask docs-check
```

**All must pass** before proceeding.

### 2. Prepare release

```bash
cargo xtask release-prepare <VERSION>

# Example:
cargo xtask release-prepare 2.5.0
```

**What this does:**
- Updates version in `Cargo.toml` files (workspace and all crates)
- Updates `CHANGELOG.md` with release notes
- Creates a release commit
- Validates version format (SemVer)

**Output:**
```
✅ Updated version to 2.5.0 in 12 files
✅ Updated CHANGELOG.md
✅ Created release commit: "chore: prepare release 2.5.0"
```

### 3. Run release verification

```bash
cargo xtask release-verify
```

**Checks (comprehensive gate):**
- ✅ Selftest passes (all 7 steps)
- ✅ Audit clean (no vulnerabilities, license compliance)
- ✅ Docs valid (`docs-check` passes)
- ✅ SBOM can be generated
- ✅ Policy compliance
- ✅ Version consistency across all Cargo.toml files
- ✅ CHANGELOG.md has entry for this version

**If verification fails:** Fix issues and re-run before proceeding.

### 4. Generate SBOM (Software Bill of Materials)

```bash
cargo xtask sbom-local
```

**Output:** `sbom.spdx.json` in repository root

**What this is:**
- Software Bill of Materials in SPDX format
- Lists all dependencies, versions, licenses
- Required for supply chain transparency
- Used in security audits

**Verify SBOM:**
```bash
# Check it was created
ls -lh sbom.spdx.json

# View summary
jq '.packages | length' sbom.spdx.json
# Should show count of all dependencies
```

### 5. Ask user for approval

**IMPORTANT:** Do NOT proceed without user approval.

```
User, I've prepared release v2.5.0:
- Version bumped in all Cargo.toml files
- CHANGELOG.md updated
- Selftest passes (11/11 steps)
- Audit clean
- SBOM generated

Ready to tag and push. Confirm to proceed?
```

**Wait for user to respond "yes" or equivalent.**

### 6. Tag the release

Only after user approval:

```bash
# Create annotated tag
git tag -a v<VERSION> -m "Release <VERSION>"

# Example:
git tag -a v2.5.0 -m "Release 2.5.0: Agent-Ready Platform Cell"
```

**Verify tag:**
```bash
git tag -n1 | grep v2.5.0
```

### 7. Push tag to remote

```bash
# Push the tag
git push origin v<VERSION>

# Example:
git push origin v2.5.0
```

**NEVER use `--force` when pushing tags to main/master.**

**What happens next:**
- CI/CD pipeline triggered (GitHub Actions)
- Full selftest runs in CI
- Provenance attestations generated
- Release artifacts built
- GitHub release created automatically

### 8. Monitor CI/CD

```bash
# Check CI status
gh run list --workflow=release.yml

# Or view in browser
open https://github.com/<org>/<repo>/actions
```

**Wait for CI to complete.** If CI fails:
- Investigate failure
- Fix if possible
- Consider rolling back tag if critical issue

## Exit Criteria

Release is successful when:
- ✅ `release-verify` passes
- ✅ SBOM generated (`sbom.spdx.json` exists)
- ✅ Git tag created (`git tag | grep vX.Y.Z`)
- ✅ Tag pushed to remote
- ✅ CI/CD completes successfully
- ✅ GitHub release published

**Then:** Release is live and ready for deployment.

## Error Handling

### If release-prepare fails:

```bash
# Common issues:
# - Invalid version format: Use SemVer (X.Y.Z)
# - Dirty working tree: Commit or stash changes
# - Version already exists: Choose a new version

# Check git status
git status

# Check existing tags
git tag | grep v
```

### If release-verify fails:

```bash
# Get detailed output
cargo xtask release-verify -v

# Common failures:
# 1. Selftest fails
#    → Use governed-governance-debug Skill
# 2. Audit fails
#    → Run: cargo xtask audit
#    → Fix vulnerabilities
# 3. Docs check fails
#    → Run: cargo xtask docs-check
#    → Fix documentation drift
```

**Fix and re-run:**
```bash
# After fixes
cargo xtask release-verify
# ✅ All checks pass - proceed
```

### If SBOM generation fails:

```bash
# Common issues:
# - cargo-sbom not installed
# - Network issues fetching metadata

# Install cargo-sbom
cargo install cargo-sbom

# Retry
cargo xtask sbom-local
```

### If CI fails after tagging:

**This is serious - release is partially published.**

```bash
# Check CI logs
gh run view --log

# If fixable:
# 1. Fix the issue
# 2. Create patch version (e.g., 2.5.1)
# 3. Re-run release process

# If critical bug:
# 1. Notify team immediately
# 2. Consider yanking the release
# 3. Document incident in friction log
```

## Examples

### Example 1: Standard release

```bash
# 1. Verify readiness
cargo xtask selftest
cargo xtask audit
# ✅ All pass

# 2. Prepare
cargo xtask release-prepare 2.5.0
# ✅ Version updated, CHANGELOG updated

# 3. Verify
cargo xtask release-verify
# ✅ All checks pass

# 4. SBOM
cargo xtask sbom-local
# ✅ sbom.spdx.json created

# 5. Ask user
# "Ready to tag v2.5.0. Confirm?"
# User: "yes"

# 6. Tag
git tag -a v2.5.0 -m "Release 2.5.0: Agent-Ready Platform Cell"

# 7. Push
git push origin v2.5.0

# 8. Monitor CI
gh run watch
# ✅ CI passes, release published
```

### Example 2: Release with pre-fix

```bash
# User: "Cut release 2.5.0"

# 1. Verify
cargo xtask selftest
# ✗ Selftest fails (AC mapping issue)

# 2. Fix governance issue
# Use governed-governance-debug Skill to fix

# 3. Re-verify
cargo xtask selftest
# ✅ All pass

# 4. Proceed with release
cargo xtask release-prepare 2.5.0
# ... continue as normal
```

### Example 3: Patch release

```bash
# Critical bug found in v2.5.0

# 1. Fix the bug (governed-feature-dev Skill)

# 2. Verify
cargo xtask selftest
# ✅ All pass

# 3. Prepare patch release
cargo xtask release-prepare 2.5.1
# ✅ Patch version bumped

# 4. Verify and release
cargo xtask release-verify
cargo xtask sbom-local
git tag -a v2.5.1 -m "Release 2.5.1: Fix critical bug in X"
git push origin v2.5.1
```

## Boundaries

**What this Skill does:**
✅ Guide release preparation and verification
✅ Validate release readiness via comprehensive gates
✅ Generate supply chain artifacts (SBOM)
✅ Create and push git tags (with user approval)

**What this Skill does NOT do:**
❌ Deploy to production (requires separate deployment workflow)
❌ Bypass governance (verification is mandatory)
❌ Handle rollbacks (requires separate incident response)
❌ Make architectural decisions (those need ADRs)

## Success Criteria

Release successful when:
- ✅ All Exit Criteria met (see above)
- ✅ Tag exists on remote
- ✅ CI/CD completed successfully
- ✅ GitHub release published with artifacts
- ✅ SBOM included in release assets
- ✅ No governance violations introduced

## References

- **Flow definition:** `specs/devex_flows.yaml` (release flow)
- **Release commands:** `docs/reference/xtask-commands.md` (release-prepare, release-verify)
- **SBOM format:** https://spdx.dev/
- **SemVer:** https://semver.org/
- **Supply chain hardening:** ADR-0006

## Notes

- **Always ask user before tagging:** Tags are immutable in most workflows
- **Never force-push to main/master:** This can break team workflows
- **Releases are governed:** release-verify is the gate
- **SBOM is required:** Part of supply chain security
- **CI must pass:** Don't manually publish releases
- **Patch releases are normal:** Better to fix quickly than delay
- **Document incidents:** Update friction log for release issues
