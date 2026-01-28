---
name: governed-maintenance
description: |
  Platform upkeep, dependency updates, policy fixes, and doc maintenance for the Rust-as-Spec platform cell. Use when fixing governance drift, updating dependencies, resolving policy violations, or maintaining documentation. Follows the maintenance flow from devex_flows.yaml.
allowed-tools:
- Read
- Grep
- Glob
- Edit
- Write
- Bash
---

# Governed Maintenance

## When to Use

Use this Skill when:
- Fixing policy violations
- Updating dependencies
- Fixing documentation drift
- Resolving friction log entries
- User says "fix the build", "update deps", "check environment health", or "fix drift"

## Prerequisites

- Repository is checked out
- Basic Rust toolchain available
- You have access to `cargo xtask` commands

## Workflow

This Skill follows the **maintenance** flow from `specs/devex_flows.yaml`.

### 1. Identify the issue

Determine what type of maintenance is needed:

```bash
# Overall health check
cargo xtask doctor

# Quick validation
cargo xtask check

# Full governance check
cargo xtask selftest
```

**Common issue types:**
- Environment health (doctor failures)
- Security vulnerabilities (audit failures)
- Policy violations (policy-test failures)
- Documentation drift (docs-check failures)
- Graph integrity (graph invariant failures)

### 2. Run targeted diagnostics

Based on the issue type:

#### Environment Health

```bash
cargo xtask doctor
```

**Checks:**
- Rust version (minimum requirement)
- Nix availability (optional but recommended)
- conftest (policy testing tool)
- Git configuration (user.name, user.email)

**Common fixes:**
- Update Rust: `rustup update`
- Install conftest: `nix profile install nixpkgs#conftest` or <https://www.conftest.dev/install/>
- Configure git: `git config --global user.name "Your Name"`

#### Security Audit

```bash
cargo xtask audit
```

**Checks:**
- Known vulnerabilities in dependencies (via cargo-audit)
- License compliance (via cargo-deny)

**Fix vulnerabilities:**

```bash
# Update specific crate
cargo update <affected-crate>

# Or update all (careful!)
cargo update

# Re-run audit
cargo xtask audit
```

#### Documentation Drift

```bash
cargo xtask docs-check
```

**Checks:**
- Doc index matches filesystem
- Front-matter valid in markdown files
- Referenced docs exist
- Version consistency

**Fix:**
- Update `specs/doc_index.yaml` to match filesystem
- Fix front-matter in markdown files
- Ensure all cross-references are valid

#### Graph Integrity

```bash
# Check graph invariants
curl http://localhost:3000/platform/graph | jq '.invariants'

# Or via selftest (step 7)
cargo xtask selftest -v
```

**Checks:**
- Requirements with `must_have_ac: true` have ACs
- Required commands are reachable via flows
- No orphaned nodes (unreachable requirements)
- All tasks reference valid ACs

**Fix:**
- Update `specs/spec_ledger.yaml` to add missing ACs
- Update `specs/devex_flows.yaml` to link commands to flows
- Remove or connect orphaned requirements

#### Policy Compliance

```bash
cargo xtask policy-test
```

**Checks:** All Rego policies pass

**Common policy violations:**
- Requirements missing `must_have_ac: true`
- ACs without test references
- Tasks without recommended flows
- Missing ADR references

**Fix:**
Edit `specs/spec_ledger.yaml` to comply with policies:

```yaml
- id: REQ-TPL-EXAMPLE
  title: "Example requirement"
  must_have_ac: true  # Add if missing
  acceptance_criteria:
    - id: AC-TPL-EX-001
      text: "Description"
      tests:  # Add if missing
        - { type: bdd, tag: "@AC-TPL-EX-001" }
```

### 3. Fix the root cause

Apply the appropriate fix based on diagnostics (see substeps above).

**General principles:**
- Fix root cause, not symptoms
- Validate fix in isolation before running full selftest
- Document non-obvious fixes in friction log

### 4. Validate the fix

Run targeted validation first:

```bash
# If fixing environment:
cargo xtask doctor

# If fixing dependencies:
cargo xtask audit

# If fixing policies:
cargo xtask policy-test

# If fixing docs:
cargo xtask docs-check
```

Then run full validation:

```bash
cargo xtask selftest
```

**Expected:** All 7 steps pass ✅

### 5. Update friction log (if applicable)

If the issue revealed a common problem or non-obvious fix:

```bash
# Add entry to friction log
cat >> docs/friction_log.md <<EOF
- $(date +%Y-%m-%d): Fixed policy violation in REQ-TPL-XYZ (missing must_have_ac flag)
  - Root cause: Policy added after requirement created
  - Fix: Added must_have_ac: true to requirement
  - Prevention: Check policy-test during AC creation
EOF
```

## Common Maintenance Tasks

### Update Dependencies

```bash
# Check for outdated crates
cargo outdated

# Update carefully (one at a time)
cargo update <crate>

# Or update all minor/patch versions
cargo update

# Verify
cargo xtask selftest
```

### Fix Formatting Drift

```bash
cargo xtask fmt-all
```

**What this does:**
- Runs `cargo fmt` on all Rust code
- Formats YAML files
- Formats TOML files
- Formats other repo artifacts

### Clean Build Artifacts

```bash
cargo xtask clean
```

**Removes:**
- `target/` directory
- `.llm/bundle/` generated files
- Temporary test artifacts

### Regenerate Hakari (workspace optimization)

```bash
cargo xtask hakari
```

**What this does:**
- Optimizes workspace dependency resolution
- Updates `workspace-hack/Cargo.toml`
- Improves build times

### Pin GitHub Actions (security)

```bash
cargo xtask pin-actions
```

**What this does:**
- Pins GitHub Actions to specific SHAs
- Prevents supply chain attacks via action updates
- Updates `.github/workflows/*.yml`

## Exit Criteria

Maintenance complete when:
- ✅ `cargo xtask doctor` passes
- ✅ `cargo xtask audit` clean (no vulnerabilities)
- ✅ `cargo xtask docs-check` passes
- ✅ `cargo xtask policy-test` passes
- ✅ `cargo xtask selftest` passes (12/12 steps)
- ✅ Specific issue resolved
- ✅ Friction log updated (if applicable)

**Then:** Environment is healthy and governance is intact.

## Error Handling

### If doctor fails

```bash
# Run doctor with verbose output
cargo xtask doctor -v

# Common issues:
# - Rust too old: rustup update
# - conftest missing: install via Nix or package manager
# - Git not configured: git config --global user.name/email
```

### If audit finds vulnerabilities

```bash
# Get details
cargo audit

# Update affected crate
cargo update <crate>

# If no update available:
# - Check if vulnerability applies to your usage
# - Consider alternative crate
# - File issue with upstream
# - Document risk acceptance in friction log
```

### If policy-test fails

```bash
# Get detailed output
cargo xtask policy-test -v

# Common fixes:
# - Add must_have_ac to requirements
# - Add test references to ACs
# - Add recommended_flows to tasks
# - Link ADRs to requirements
```

### If docs-check fails

```bash
# Get details
cargo xtask docs-check

# Common fixes:
# - Update doc_index.yaml to match filesystem
# - Fix YAML front-matter in markdown files
# - Remove references to deleted docs
```

## Examples

### Example 1: Update dependency

```bash
# User: "Update axum to latest version"

# 1. Check current version
grep axum Cargo.toml

# 2. Update
cargo update axum

# 3. Verify
cargo xtask check
cargo xtask audit
cargo xtask selftest

# 4. Document
echo "- $(date +%Y-%m-%d): Updated axum to 0.7.x for security patch" >> docs/friction_log.md
```

### Example 2: Fix policy violation

```bash
# Selftest fails: "REQ-TPL-XYZ missing must_have_ac"

# 1. Identify issue
cargo xtask policy-test
# Output: REQ-TPL-XYZ must have must_have_ac: true

# 2. Fix
# Edit specs/spec_ledger.yaml:
# - id: REQ-TPL-XYZ
#   must_have_ac: true  # Add this line

# 3. Verify
cargo xtask policy-test
cargo xtask selftest
# ✅ All pass
```

### Example 3: Fix environment

```bash
# User reports: "xtask commands failing"

# 1. Diagnose
cargo xtask doctor
# Output: ✗ conftest not found

# 2. Fix
nix profile install nixpkgs#conftest

# 3. Verify
cargo xtask doctor
# ✅ All checks pass

# 4. Full validation
cargo xtask selftest
# ✅ All pass
```

## Boundaries

**What this Skill does:**
✅ Diagnose environment and governance issues
✅ Fix policy violations and documentation drift
✅ Update dependencies safely
✅ Maintain platform health

**What this Skill does NOT do:**
❌ Make breaking architectural changes (need ADR via `adr-new`)
❌ Bypass selftest (it's the contract)
❌ Introduce new features (use `governed-feature-dev` Skill)
❌ Cut releases (use `governed-release` Skill)

## Success Criteria

Maintenance successful when:
- ✅ All Exit Criteria met (see above)
- ✅ No new drift introduced
- ✅ Root cause identified and fixed
- ✅ Friction log updated with learnings

## References

- **Flow definition:** `specs/devex_flows.yaml` (maintenance flow)
- **Policy definitions:** `policies/*.rego`
- **Audit config:** `.cargo/audit.toml`, `deny.toml`
- **Doc index:** `specs/doc_index.yaml`
- **xtask reference:** `docs/reference/xtask-commands.md`

## Notes

- **Maintenance is continuous:** Not a one-time task
- **Doctor is fast:** Run it anytime something feels off
- **Audit is critical:** Run before every release
- **Policies enforce invariants:** Don't bypass them, fix the issue
- **Friction log captures learnings:** Document non-obvious fixes
