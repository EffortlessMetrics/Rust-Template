# Governed Maintenance

**Skill:** governed-maintenance  
**Purpose:** Diagnose and fix environment/governance drift issues

---

## When to Use This Skill

Use this skill when asked to:
- "Fix the build"
- "Diagnose issues"
- "Check environment health"
- "Update dependencies"
- "Fix drift"

---

## Diagnostic Commands

### 1. Environment Health

```bash
cargo xtask doctor
```

**Checks:**
- Rust version
- Nix availability
- conftest (policy testing)
- Git configuration

**Action:** Fix any failures reported.

### 2. Security Audit

```bash
cargo xtask audit
```

**Checks:** Known vulnerabilities in dependencies

**If vulnerabilities found:**
```bash
cargo update <affected-crate>
cargo xtask audit  # Verify fix
```

### 3. Documentation Drift

```bash
cargo xtask docs-check
```

**Checks:**
- Doc index matches filesystem
- Front-matter valid
- Referenced docs exist

**Fix:** Update `specs/doc_index.yaml` or fix docs.

### 4. Graph Integrity

```bash
cargo xtask graph-export --check-invariants
```

**Checks:**
- Requirements with `must_have_ac` have ACs
- Required commands are reachable via flows
- No orphaned nodes

**Fix:** Update `specs/spec_ledger.yaml` or `specs/devex_flows.yaml`.

### 5. Policy Compliance

```bash
cargo xtask policy-test
```

**Checks:** All Rego policies pass

**Fix:** Update specs or code to match policies.

---

## Common Maintenance Tasks

### Update Dependencies

```bash
# Check for outdated crates
cargo outdated

# Update carefully
cargo update <crate>

# Verify
cargo xtask selftest
```

### Fix Formatting Drift

```bash
cargo xtask fmt-all
```

### Clean Build Artifacts

```bash
cargo xtask clean
```

### Regenerate Hakari (workspace optimization)

```bash
cargo xtask hakari
```

---

## Workflow Example

```bash
# 1. User reports: "Build is broken"

# 2. Run diagnostics
cargo xtask doctor
cargo xtask selftest

# 3. Read error output
# Example: "[7/7] Graph invariants failed"

# 4. Drill down
cargo xtask graph-export --check-invariants
# Output: "Command 'new-cmd' in flow 'my-flow' not defined"

#5. Fix root cause
# Add command to devex_flows.yaml under 'commands:'

# 6. Verify
cargo xtask selftest
# ✅ All pass
```

---

## Boundaries

**What this skill does:**
✅ Diagnose environment issues  
✅ Fix governance drift  
✅ Update dependencies safely

**What this skill does NOT do:**
❌ Make breaking architectural changes (need ADR)  
❌ Bypass selftest (it's the contract)  
❌ Introduce new features (use `governed-feature-dev`)

---

## Success Criteria

Maintenance complete when:
- ✅ `cargo xtask doctor` passes
- ✅ `cargo xtask audit` clean
- ✅ `cargo xtask docs-check` passes
- ✅ `cargo xtask selftest` passes

**Then:** Environment is healthy.
