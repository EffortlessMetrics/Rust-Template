# Fix AC-PLT-021: service-init command

## AC Definition

From `specs/spec_ledger.yaml` (lines 135-144):

```yaml
AC-PLT-021:
  text: "`cargo xtask service-init` updates service_metadata.yaml,
         README, and CLAUDE.md with a new service ID, name, and
         description, and `/platform/status` reflects the new identity."
  adr: ADR-0022
  tags: [kernel]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-PLT-021", file: "specs/features/xtask_devex.feature" }
```

## Current Status

**Implementation: 80% complete**

### ✅ What Works

- Command exists: `cargo xtask service-init`
- Updates `service_metadata.yaml` with service_id, display_name, description
- Updates `README.md` with new service name and description
- ID validation (kebab-case regex)
- Idempotent operation (safe to run multiple times)
- Comprehensive unit tests (6 test functions)
- BDD test infrastructure (5 scenarios)

### ❌ What's Missing

1. **CLAUDE.md update NOT implemented** (spec requirement)
   - Spec says: "updates README, and CLAUDE.md"
   - Current state: NO CODE implements CLAUDE.md updates
   - File: `crates/xtask/src/commands/service_init.rs` missing `update_claude()` function

2. **Command NOT registered in devex_flows.yaml** (governance contract violation)
   - All other commands are listed in `specs/devex_flows.yaml`
   - `service-init` is missing from the flows spec
   - Selftest will flag this as "devex contract not satisfied"

## BDD Test Scenarios

From `specs/features/xtask_devex.feature` (lines 567-612):

1. ✅ Basic branding update (metadata + README)
2. ✅ Idempotency test (second run returns "No changes needed")
3. ✅ ID format validation (rejects non-kebab-case IDs)
4. ✅ Service identity update for new services
5. ⚠️ Platform status reflection (will fail on CLAUDE.md assertion)

## Files Requiring Changes

### Priority 1 (Required)

1. **`crates/xtask/src/commands/service_init.rs`**
   - Add `update_claude()` function (similar to `update_readme()` pattern)
   - Extract and replace service name in CLAUDE.md
   - Handle version suffix preservation
   - Call it in `run()` orchestration (around line 51)

2. **`specs/devex_flows.yaml`**
   - Add `service-init:` entry under `commands:` section
   - Example:

     ```yaml
     service-init:
       category: service_setup
       summary: "Initialize service branding (ID, name, description)"
       required: true
       docs:
         readme_table: true
         contributing_flow: false
         claude_golden_path: true
     ```

3. **`crates/acceptance/src/steps/xtask_devex.rs`**
   - Add CLAUDE.md backup tracking in service-init setup steps
   - Add cleanup for CLAUDE.md restoration (around line 2144 restore logic)

4. **`specs/features/xtask_devex.feature`**
   - Add CLAUDE.md assertion to scenarios (around line 575-577):

     ```gherkin
     And "CLAUDE.md" should contain "My New Service"
     ```

## Verification Commands

```bash
# Run targeted BDD tests
cargo xtask bdd --tags @AC-PLT-021

# Run AC-specific tests
cargo xtask ac-tests AC-PLT-021

# Verify AC status
cargo xtask ac-status | grep AC-PLT-021

# Full governance check
cargo xtask selftest
```

## Acceptance Criteria

- [ ] `update_claude()` function implemented in service_init.rs
- [ ] `update_claude()` called in `run()` function
- [ ] CLAUDE.md backup/restore in BDD steps
- [ ] `service-init` registered in devex_flows.yaml
- [ ] BDD scenario includes CLAUDE.md assertion
- [ ] All 5 BDD scenarios pass
- [ ] `cargo xtask selftest` passes AC-PLT-021

## Related Files

- Implementation: `crates/xtask/src/commands/service_init.rs` (336 lines)
- Spec: `specs/spec_ledger.yaml` (lines 135-144)
- BDD: `specs/features/xtask_devex.feature` (lines 567-612)
- Steps: `crates/acceptance/src/steps/xtask_devex.rs` (lines 2050-2300)
- Flows: `specs/devex_flows.yaml` (392 lines, missing service-init)
- ADR: `docs/adr/0022-platform-metadata-and-test-isolation.md`

## Labels

`kernel`, `ac-fail`, `must-fix`, `xtask`, `governance`

---

## Failure Analysis (2025-12-02)

### Test Execution Summary

**Command:** `CUCUMBER_TAG_EXPRESSION="@AC-PLT-021" cargo test -p acceptance --test acceptance`

**Results:** 2 of 5 scenarios FAILED

```
5 scenarios (3 passed, 2 failed)
27 steps (25 passed, 2 failed)
```

### Failed Scenarios

#### 1. Scenario: "service-init updates service branding" (Line 568)

**Location:** `specs/features/xtask_devex.feature:568-579`

**Failed Step:**

```gherkin
And "specs/service_metadata.yaml" should contain "service_id: test-service"
```

**Error Message:**

```
Step panicked. Captured output: Expected 'specs/service_metadata.yaml' to contain 'service_id: test-service'
Actual content:
service_id: my-new-service
display_name: My New Service
description: A new test service
```

**Root Cause:** Test state isolation issue. The BDD scenarios are not properly isolated - the first scenario's changes persist into subsequent scenarios, causing assertion mismatches.

**Expected Behavior:** Each scenario should start with a clean state. When scenario 1 sets `test-service`, scenario 1 assertions should see `test-service`.

**Actual Behavior:** The test sees state from a different scenario (`my-new-service` from scenario 4), indicating shared mutable state between test runs.

#### 2. Scenario: "service-init updates metadata and README for new service identity" (Line 596)

**Location:** `specs/features/xtask_devex.feature:596-606`

**Failed Step:**

```gherkin
And "specs/service_metadata.yaml" should contain "display_name: My New Service"
```

**Error Message:**

```
Step panicked. Captured output: Expected 'specs/service_metadata.yaml' to contain 'display_name: My New Service'
Actual content:
service_id: test-service
display_name: Test Service
description: A test service
```

**Root Cause:** Same test isolation issue - this scenario expects to see `My New Service` but instead sees `Test Service` from a previous scenario.

### Implementation Status

#### ✅ What Works (Implementation Complete)

1. **CLAUDE.md Update Function Implemented**
   - Function `update_claude()` exists at lines 247-318 in `crates/xtask/src/commands/service_init.rs`
   - Properly extracts version suffix from title
   - Updates title to format: `# CLAUDE.md – {name} {version_suffix}`
   - Correctly called in `run()` at line 54

2. **Core Functionality**
   - Service ID validation (kebab-case regex)
   - Metadata update (service_id, display_name, description)
   - README update (title and description)
   - CLAUDE.md update (title)
   - Idempotency checks (no-op when values unchanged)
   - Fork registry placeholder

3. **Test Infrastructure**
   - 6 unit tests for ID validation (all passing)
   - 5 BDD scenarios defined
   - Test steps implemented in `crates/acceptance/src/steps/xtask_devex.rs`

#### ❌ What's Broken (Test Harness Issue)

**The BDD test harness has a critical state isolation bug:**

1. **No proper cleanup between scenarios**
   - Each scenario should backup original files before modification
   - Each scenario should restore original files after completion
   - Currently: file modifications persist across scenario boundaries

2. **Backup/Restore Implementation Missing**
   - The acceptance step code in `crates/acceptance/src/steps/xtask_devex.rs` needs:
     - `backup_service_files()` function to save original state
     - `restore_service_files()` function to reset state after each scenario
     - Proper Given/After hooks to ensure isolation

3. **Race Condition Between Scenarios**
   - Scenario 1 writes "test-service" → Scenario 4 writes "my-new-service"
   - Scenario 1's assertions may run AFTER scenario 4's writes if not properly isolated
   - This explains why scenario 1 sees "my-new-service" instead of "test-service"

### Commands to Reproduce

```bash
# Run the failing scenarios
CUCUMBER_TAG_EXPRESSION="@AC-PLT-021" cargo test -p acceptance --test acceptance

# Run with verbose output
CUCUMBER_TAG_EXPRESSION="@AC-PLT-021" RUST_LOG=debug cargo test -p acceptance --test acceptance -- --nocapture

# Check current state after test run
cat specs/service_metadata.yaml
```

### What Needs to be Fixed

#### Priority 1: Fix Test Isolation

**File:** `crates/acceptance/src/steps/xtask_devex.rs`

1. **Add backup mechanism in Given steps:**

   ```rust
   // Before each @AC-PLT-021 scenario starts:
   - Backup specs/service_metadata.yaml
   - Backup README.md
   - Backup CLAUDE.md
   ```

2. **Add restore mechanism in After hooks:**

   ```rust
   // After each @AC-PLT-021 scenario completes:
   - Restore specs/service_metadata.yaml from backup
   - Restore README.md from backup
   - Restore CLAUDE.md from backup
   ```

3. **Ensure proper Given step ordering:**

   ```gherkin
   Given a clean development environment  # Must include file backup
   Given a clean git working directory    # Ensures no uncommitted changes
   ```

#### Priority 2: Verify Devex Flows Registration

**File:** `specs/devex_flows.yaml`

Check if `service-init` is registered in the flows spec. If missing, add:

```yaml
commands:
  service-init:
    category: service_setup
    summary: "Initialize service branding (ID, name, description)"
    required: true
    docs:
      readme_table: true
      contributing_flow: false
      claude_golden_path: true
```

### Expected vs Actual Behavior Summary

| Aspect | Expected | Actual |
|--------|----------|--------|
| Test Isolation | Each scenario has clean slate | Scenarios share mutable state |
| Scenario 1 Assertion | Sees "test-service" | Sees "my-new-service" (from scenario 4) |
| Scenario 4 Assertion | Sees "My New Service" | Sees "Test Service" (from scenario 1) |
| CLAUDE.md Update | Implemented ✅ | Function exists and is called |
| Test Pass Rate | 5/5 scenarios pass | 3/5 scenarios pass (2 fail due to isolation) |

### Current AC Status

**Status:** FAIL (due to test harness isolation bug, not implementation bug)

The implementation of AC-PLT-021 is functionally complete:
- ✅ Updates service_metadata.yaml
- ✅ Updates README.md
- ✅ Updates CLAUDE.md
- ✅ Validates service ID format
- ✅ Idempotent operation
- ✅ Platform status reflects changes (when tests run in isolation)

The failure is in the **test harness**, not the **production code**. The acceptance tests need proper state isolation to correctly validate the working implementation.

### Verification After Fix

Once test isolation is fixed, verify with:

```bash
# Run AC-specific tests
cargo xtask ac-tests AC-PLT-021

# Run targeted BDD
CUCUMBER_TAG_EXPRESSION="@AC-PLT-021" cargo test -p acceptance --test acceptance

# Verify AC status
cargo xtask ac-status | grep AC-PLT-021

# Full governance check
cargo xtask selftest
```

### Related ADRs

- **ADR-0022:** Platform Metadata and Test Isolation (should document test isolation requirements)
- **ADR-0005:** Selftest as single gate (AC-PLT-021 is kernel gate)
- **ADR-0003:** Spec and BDD as source of truth (failing due to test harness issue)
