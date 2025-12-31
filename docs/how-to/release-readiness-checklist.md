---
id: HOW-TO-RELEASE-READINESS
title: "Release Readiness Checklist"
doc_type: how_to
version: 3.3.14
stories: [US-TPL-PLT-001]
requirements:
  - REQ-PLT-RELEASE-SAFETY
  - REQ-PLT-SECURITY-GOVERNANCE
acs:
  - AC-PLT-019
  - AC-PLT-020
last_updated: 2025-12-27
---

# Release Readiness Checklist

This checklist ensures a new release is **fully validated, locked down, and ready to ship**.

---

## Pre-Release Verification

### 1. Local Governance Gates

Run these commands locally before cutting a release:

```bash
# Enter Nix shell (Tier 1 canonical environment)
nix develop

# Fast governance check
cargo xtask check

# Full governance gate (all 11 selftest steps)
KERNEL_UNKNOWN_BUDGET=0 cargo xtask selftest

# Documentation governance
cargo xtask docs-check
```

**Expected outcome:** All commands pass with no errors.

### 2. CI Status on `main`

Verify the following CI checks are green on `main`:

| Check | Workflow | Required |
|-------|----------|----------|
| `selftest` | `tier1-selftest.yml` | **Yes** |
| `Tier 1 (macOS + Nix)` | `ci-template-selftest.yml` | **Yes** |
| `Tier 2 (Windows native)` | `ci-template-selftest.yml` | **Yes** |

**How to verify:**
- Go to the [Actions tab](../../actions) on GitHub
- Check the most recent `main` branch run for each workflow
- All must show green checkmarks

### 3. AC Coverage Status

```bash
cargo xtask ac-status
cargo xtask ac-coverage --must-have
```

**Expected outcome:**
- All kernel ACs (must_have_ac=true) have PASS status
- No UNKNOWN or FAIL kernel ACs

---

## Release Preparation

### 4. Bump Version

```bash
cargo xtask release-prepare X.Y.Z
```

This command:
- Updates `Cargo.toml` version
- Updates `specs/spec_ledger.yaml` template_version
- Updates version references across the codebase
- Regenerates `docs/feature_status.md`

### 5. Update CHANGELOG

Edit `CHANGELOG.md` to document changes:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

**Checklist for CHANGELOG entry:**
- [ ] Version matches what `release-prepare` set
- [ ] Date is today
- [ ] All notable changes are documented
- [ ] Breaking changes are clearly marked (if any)

### 6. Re-validate After Version Bump

```bash
cargo xtask check
cargo xtask selftest
```

---

## Tag and Push

### 7. Create Release Commit

```bash
git add -A
git commit -m "release: vX.Y.Z"
```

### 8. Create Annotated Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 9. Push Branch and Tag

```bash
git push origin main
git push origin vX.Y.Z
```

---

## Post-Release Verification

### 10. Verify Tag Workflows

After pushing the tag, verify these workflows run and pass:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `release-sbom-sign.yml` | `v*` tags | Generate SBOM, sign artifacts |
| `ci-supply-chain.yml` | `v*` tags | Build provenance attestation |
| `ci-example-fork.yml` | `v*` tags | Full selftest (release gate) |

**How to verify:**
- Go to the [Actions tab](../../actions) on GitHub
- Filter by the new tag
- All tag-triggered workflows should be green

### 11. Verify Release Artifacts

After workflows complete, verify artifacts exist:

- [ ] GitHub Release created (if using releases)
- [ ] `sbom.json` attached to release
- [ ] Build attestations visible in GitHub Security tab

---

## Branch Protection Requirements

Ensure these are configured in **Settings > Branches > main**:

### Required Status Checks

| Check Name | Source Workflow |
|------------|-----------------|
| `selftest` | `tier1-selftest.yml` |
| `Template Self-Test / Tier 1 (macOS + Nix)` | `ci-template-selftest.yml` |
| `Template Self-Test / Tier 2 (Windows native)` | `ci-template-selftest.yml` |

### Recommended Settings

- [x] **Require branches to be up to date before merging**
- [x] **Require conversation resolution before merging**
- [ ] **Include administrators** (optional, for strict lock-down)

### Check for Stale/Removed Checks

If PRs are blocked by "Expected — Waiting for status to be reported":
1. Go to Settings > Branches > main protection rule
2. Look for required checks that no longer exist (renamed/removed workflows)
3. Remove stale checks, add the correct current check names

---

## Tiering Model Reference

The `tier1-selftest.yml` workflow uses path-based classification:

| Tier | Trigger Condition | What Runs |
|------|-------------------|-----------|
| **docs-check** | Only `docs/**`, `*.md` changed | `cargo xtask docs-check` |
| **check** | Non-BDD changes (workflows, configs) | `cargo xtask check` |
| **selftest** | `crates/**`, `specs/**`, `*.feature`, `Cargo.*` | Full `cargo xtask selftest` |

**On release tags:** `ci-example-fork.yml` always runs full selftest regardless of path filters.

---

## Quick Reference Commands

```bash
# Pre-release validation
nix develop -c cargo xtask docs-check
nix develop -c cargo xtask check
KERNEL_UNKNOWN_BUDGET=0 nix develop -c cargo xtask selftest

# Release preparation
cargo xtask release-prepare X.Y.Z
# Edit CHANGELOG.md
cargo xtask selftest

# Tag and push
git add -A && git commit -m "release: vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main && git push origin vX.Y.Z
```

---

## Troubleshooting

### CI is red but I haven't changed anything

1. Check if a scheduled workflow (nightly) failed
2. Check if a dependency was flagged by `cargo audit`
3. Run `cargo xtask audit` locally to see vulnerabilities

### Required check is "Expected — Waiting for status"

The required check name doesn't match any workflow's check output. Either:
- The workflow was renamed/removed
- The check name was changed
- Update branch protection to use the correct check name

### Release tag workflow failed

1. Check the workflow logs for the specific error
2. Common issues:
   - Missing secrets (check repository secrets configuration)
   - SBOM generation failed (check `cargo-cyclonedx` is available)
   - Attestation failed (check `id-token: write` permission)

---

## See Also

- [CI Workflows Reference](../reference/ci-workflows.md) - Detailed workflow documentation
- [Governed Release Skill](../../.claude/skills/governed-release/SKILL.md) - Agent workflow for releases
- [ADR-0017: Tier-1 Selftest Gate](../adr/0017-tier1-selftest-gate.md) - Rationale for tiered CI
