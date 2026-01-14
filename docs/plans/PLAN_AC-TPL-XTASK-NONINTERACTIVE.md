# Plan: AC-TPL-XTASK-NONINTERACTIVE - Noninteractive Mode

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-TPL-XTASK-NONINTERACTIVE

## Scope

**Crates/modules:**
- `crates/xtask/src/commands/*.rs` - Add noninteractive mode detection to all covered commands
- `crates/xtask/src/lib.rs` or `crates/xtask/src/util.rs` (new) - Shared environment detection helper
- `specs/features/xtask_devex.feature` - BDD scenarios for noninteractive mode
- `crates/acceptance/src/steps/xtask_devex.rs` - BDD step implementations

**Commands covered by DevEx contract:**
- `doctor`, `check`, `selftest`, `ac-status`, `ac-coverage`, `bundle`, `version`
- `friction-list`, `friction-new`, `questions-list`, `fork-list`

## Goals

1. Implement environment variable detection: `CI=1` or `XTASK_NONINTERACTIVE=1`
2. Guarantee no interactive prompts in noninteractive mode
3. Ensure stable exit codes: 0 on success, non-zero on failure
4. Validate behavior via BDD tests for all covered commands

## Implementation Steps

1. **Create shared environment detection helper**
   - Add `crates/xtask/src/util.rs` (or extend existing util module)
   - Create function:

     ```rust
     pub fn is_noninteractive() -> bool {
         std::env::var("CI").is_ok() ||
         std::env::var("XTASK_NONINTERACTIVE").is_ok()
     }
     ```

   - Expose from `lib.rs`: `pub mod util;`

2. **Audit and fix all covered commands**
   - For each command in the DevEx contract list:
     - Search for interactive prompts (stdin reads, `dialoguer`, confirmation prompts)
     - Wrap interactive code with `if !is_noninteractive() { ... }`
     - Provide sensible defaults or fail fast in noninteractive mode
   - Specific commands to check:
     - `doctor`: Skip interactive remediation suggestions
     - `check`: Already noninteractive (verify)
     - `selftest`: Already noninteractive (verify)
     - `ac-status`: Already noninteractive (verify)
     - `bundle`: Skip any interactive prompts for task selection
     - `friction-new`: Require all args via flags in noninteractive mode
     - Others: Verify and document behavior

3. **Document noninteractive mode behavior**
   - Add doc comments to each affected command's `run()` function
   - Example:

     ```rust
     /// In noninteractive mode (CI=1 or XTASK_NONINTERACTIVE=1),
     /// this command skips all prompts and uses default values.
     ```

4. **Ensure stable exit codes**
   - Verify each command returns `Ok(())` on success (exit 0)
   - Verify each command returns `Err(...)` on failure (exit non-zero)
   - No commands should return 0 on failure or non-zero on success

5. **Write BDD scenarios** (`specs/features/xtask_devex.feature`)
   - Scenario 1: "Commands respect CI=1 environment variable"
     - Tag: `@AC-TPL-XTASK-NONINTERACTIVE`
     - Steps: set CI=1, run doctor/check/selftest, verify no prompts, check exit code
   - Scenario 2: "Commands respect XTASK_NONINTERACTIVE=1 environment variable"
     - Tag: `@AC-TPL-XTASK-NONINTERACTIVE`
     - Steps: set XTASK_NONINTERACTIVE=1, run commands, verify behavior
   - Scenario 3: "Noninteractive commands exit with correct status"
     - Tag: `@AC-TPL-XTASK-NONINTERACTIVE`
     - Steps: run command in CI mode, verify exit 0 on success, non-zero on failure

6. **Implement BDD steps** (`crates/acceptance/src/steps/xtask_devex.rs`)
   - Add step: `Given the environment variable "CI" is set to "1"`
   - Add step: `Given the environment variable "XTASK_NONINTERACTIVE" is set to "1"`
   - Add step: `Then the command should not prompt for input`
   - Add step: `And the exit code should be 0`
   - Add step: `And the exit code should be non-zero`

## Verification Commands

```bash
# Manual verification
CI=1 cargo xtask doctor
CI=1 cargo xtask check
XTASK_NONINTERACTIVE=1 cargo xtask selftest

# Check exit codes
CI=1 cargo xtask check && echo "Success (exit 0)" || echo "Failed (non-zero)"

# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-TPL-XTASK-NONINTERACTIVE" cargo test -p acceptance --test acceptance

# Verify AC status
cargo xtask ac-status | grep AC-TPL-XTASK-NONINTERACTIVE

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] `is_noninteractive()` helper function created in util.rs
- [ ] All DevEx contract commands audited for interactive prompts
- [ ] Interactive code wrapped with `if !is_noninteractive()` guards
- [ ] Exit codes verified: 0 on success, non-zero on failure
- [ ] BDD scenarios written and tagged @AC-TPL-XTASK-NONINTERACTIVE
- [ ] BDD steps implemented in xtask_devex.rs
- [ ] All @AC-TPL-XTASK-NONINTERACTIVE tests pass
- [ ] Manual test: `CI=1 cargo xtask doctor` completes without prompts
- [ ] Manual test: `XTASK_NONINTERACTIVE=1 cargo xtask check` returns correct exit code
- [ ] `cargo xtask ac-status` shows AC-TPL-XTASK-NONINTERACTIVE as PASS
- [ ] No other ACs flip to FAIL

## Notes

- **Priority:** High - blocks CI and agent automation
- **Risk Level:** Low-Medium - changes are additive (wrapping existing code)
- **Testing Strategy:** BDD scenarios + manual verification in CI environment
- **Compatibility:** Backward compatible - interactive mode unchanged by default
