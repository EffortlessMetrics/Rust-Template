# Governed Release

**Skill:** governed-release  
**Purpose:** Prepare and execute a release following governance contracts

---

## When to Use This Skill

Use this skill when asked to:
- "Cut a release"
- "Prepare version X.Y.Z"
- "Tag a new version"
- "Deploy to production"

---

## Prerequisites

- All features are complete (`cargo xtask selftest` passes)
- CHANGELOG.md is up to date
- No outstanding policy violations

---

## Workflow Steps

### 1. Prepare Release

```bash
cargo xtask release-prepare <VERSION>
# Example: cargo xtask release-prepare 2.5.0
```

**What this does:**
- Updates version in `Cargo.toml` files
- Updates `CHANGELOG.md` with release notes
- Creates a release commit

### 2. Verify Release Readiness

```bash
cargo xtask release-verify
```

**Checks:**
- ✅ Selftest passes
- ✅ Audit clean (no vulnerabilities)
- ✅ Docs valid (`docs-check`)
- ✅ SBOM generated
- ✅ Policy compliance

**If verification fails:** Fix issues before proceeding.

### 3. Generate SBOM

```bash
cargo xtask sbom-local
```

**Output:** `sbom.spdx.json`

###4. Tag the Release

```bash
git tag -a v<VERSION> -m "Release <VERSION>"
git push origin v<VERSION>
```

**CI will:**
- Run full selftest
- Generate provenance attestations
- Publish release artifacts

---

## Example Execution

```bash
# 1. Prepare
cargo xtask release-prepare 2.5.0

# 2. Verify
cargo xtask release-verify
# If passes:

# 3. SBOM
cargo xtask sb om-local

# 4. Tag
git tag -a v2.5.0 -m "Release 2.5.0: Agent-Ready Platform Cell"
git push origin v2.5.0

# 5. CI takes over (GitHub Actions)
```

---

## Boundaries

**What this skill does:**
✅ Guide release preparation  
✅ Validate release readiness  
✅ Generate supply chain artifacts

**What this skill does NOT do:**
❌ Deploy to production (requires human approval)  
❌ Bypass governance (verification is mandatory)  
❌ Handle rollbacks (requires separate workflow)

---

## Error Recovery

**If `release-verify` fails:**

1. **Read the error:**
   ```
   ✗ Audit failed: 2 vulnerabilities found
   ```

2. **Fix:**
   ```bash
   cargo xtask audit
   cargo update <crate>
   ```

3. **Re-verify:**
   ```bash
   cargo xtask release-verify
   ```

---

## Success Criteria

Release is ready when:
- ✅ `release-verify` passes
- ✅ SBOM generated
- ✅ Tag pushed
- ✅ CI completes successfully

**Then:** Release is live.
