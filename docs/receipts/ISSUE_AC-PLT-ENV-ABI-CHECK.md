# Fix AC-PLT-ENV-ABI-CHECK: env ABI detection

## AC Definition

From `specs/spec_ledger.yaml` (lines 156-164):

```yaml
AC-PLT-ENV-ABI-CHECK:
  text: "`cargo xtask doctor` detects ABI mismatches between system rustc
         and Nix devshell rustc, warns about proc-macro compatibility issues,
         and points to TROUBLESHOOTING.md for resolution."
  requirement: REQ-PLT-ENV-DIAGNOSTICS
  tags: [kernel]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-PLT-ENV-ABI-CHECK", file: "specs/features/xtask_devex.feature" }
```

## Current Status

**Implementation: 90% complete, 0% passing tests**

### ✅ What Works (Implementation)

- ✅ ABI mismatch detection (compares system vs Nix rustc)
- ✅ proc-macro compatibility warnings
- ✅ References TROUBLESHOOTING.md §rust-analyzer ABI
- ✅ glibc compatibility check (Linux only)
- ✅ libz.so.1 availability check
- ✅ Structured output sections
- ✅ Exit codes correctly classify warnings vs errors
- ✅ Unit test coverage in doctor.rs

### ❌ What's Missing (BDD Test Infrastructure)

**4 BDD step definitions are referenced but NOT implemented:**

1. **Pattern: "the output should mention either X or Y"** (line 83 in xtask_devex.feature)
2. **Step: "the output should show ABI check result"** (line 91)
3. **Step: "the output should show glibc status"** (line 99)
4. **Pattern: "if warnings exist then output should mention"** (line 121)

## BDD Test Scenarios

From `specs/features/xtask_devex.feature` (lines 67-128):

### Test Execution Status

```
✅ Scenario 1: doctor shows structured sections (passes)
❌ Scenario 2: doctor detects environment type (blocked by missing step)
❌ Scenario 3: doctor checks ABI compatibility (blocked by missing step)
❌ Scenario 4: doctor checks glibc version (blocked by missing step)
✅ Scenario 5: doctor checks libz.so.1 (passes)
✅ Scenario 6: doctor reports exit code (passes)
```

**AC Status:** 4/6 scenarios blocked → AC fails

## Files Requiring Changes

### Priority 1 (Required)

**Only one file needs changes:**

**`crates/acceptance/src/steps/xtask_devex.rs`**

Add 4 missing step definitions:

```rust
// Pattern 1: Either/or assertion
#[then(regex = r#"^the output should mention either "([^"]+)" or "([^"]+)"$"#)]
async fn then_output_mention_either(world: &mut World, option1: String, option2: String) {
    let output = world.last_command_output.as_ref().unwrap();
    assert!(
        output.contains(&option1) || output.contains(&option2),
        "Output should mention either '{}' or '{}', but got:\n{}",
        option1, option2, output
    );
}

// Pattern 2: Show ABI check result
#[then("the output should show ABI check result")]
async fn then_output_shows_abi_result(world: &mut World) {
    let output = world.last_command_output.as_ref().unwrap();
    let is_match_or_mismatch = output.contains("match") || output.contains("mismatch");
    assert!(
        is_match_or_mismatch,
        "Output should show ABI check result (match/mismatch)\nActual:\n{}",
        output
    );
}

// Pattern 3: Show glibc status
#[then("the output should show glibc status")]
async fn then_output_shows_glibc_status(world: &mut World) {
    let output = world.last_command_output.as_ref().unwrap();
    let is_valid = output.contains("glibc") || output.contains("N/A") || output.contains("libc");
    assert!(
        is_valid,
        "Output should show glibc status\nActual:\n{}",
        output
    );
}

// Pattern 4: Conditional warning check
#[then(regex = r#"^if warnings exist then output should mention "([^"]+)"$"#)]
async fn then_conditional_warning_mention(world: &mut World, text: String) {
    let output = world.last_command_output.as_ref().unwrap();
    if output.contains("⚠") {
        assert!(
            output.contains(&text),
            "Output contains warnings but doesn't mention '{}'\nActual:\n{}",
            text, output
        );
    }
}
```

### No Other Changes Required

- ✅ `crates/xtask/src/commands/doctor.rs` - Implementation complete
- ✅ `specs/features/xtask_devex.feature` - Feature file correct
- ✅ `docs/TROUBLESHOOTING.md` - References correct

## Root Cause

The `doctor` command implementation is **complete and working**. The BDD test scenarios reference step definitions that **do not exist** in the acceptance test harness. This is purely a test infrastructure gap.

## Verification Commands

```bash
# After adding step definitions:
cargo xtask test-ac AC-PLT-ENV-ABI-CHECK

# Verify AC status
cargo xtask ac-status | grep AC-PLT-ENV-ABI-CHECK

# Full governance check
cargo xtask selftest
```

## Acceptance Criteria

- [ ] Add 4 missing step definitions to xtask_devex.rs
- [ ] All 6 BDD scenarios pass
- [ ] `cargo xtask ac-status` shows AC-PLT-ENV-ABI-CHECK as passing
- [ ] `cargo xtask selftest` passes AC-PLT-ENV-ABI-CHECK

## Related Files

- BDD Steps: `crates/acceptance/src/steps/xtask_devex.rs` (needs 4 step definitions)
- Doctor Implementation: `crates/xtask/src/commands/doctor.rs:47-401` (complete ✅)
- BDD Scenarios: `specs/features/xtask_devex.feature:67-128`
- Spec: `specs/spec_ledger.yaml:156-164`
- Docs: `docs/TROUBLESHOOTING.md:294-348`

## Estimated Effort

**15 minutes** - Add 4 straightforward step definitions

## Labels

`kernel`, `ac-fail`, `must-fix`, `test-infrastructure`, `bdd`
