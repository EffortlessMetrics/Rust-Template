<!-- doclint:disable orphan-version -->
# BDD Implementation - Batch 2: JSON Validation and Environment Variables

**Date:** 2025-12-02
**Template Version:** 3.3.6
**Implemented By:** Claude (Autonomous Agent)
**Status:** ✅ Complete and Tested

---

## Overview

Batch 2 adds comprehensive JSON validation capabilities and environment variable management to the BDD test framework. These steps enable:

- Advanced JSON field validation (nested fields, field existence checks)
- JSON file content validation
- Environment variable setup for test scenarios
- Improved test isolation and configurability

---

## Implemented Steps (4 total)

### 1. JSON Field Validation Steps

#### `the JSON should contain field "<field_name>"`

**Purpose:** Verifies that a JSON output from a CLI command contains a specific top-level field.

**Location:** `../../crates/acceptance/src/steps/xtask_devex.rs` (lines 2850-2869)

**Usage Example:**

```gherkin
When I run "cargo xtask idp-snapshot"
Then the output should be valid JSON
And the JSON should contain field "timestamp"
And the JSON should contain field "template_version"
And the JSON should contain field "governance_health"
```

**Implementation Details:**
- Extracts JSON from command output (skipping Nix/cargo messages)
- Parses JSON and validates it's an object
- Checks for field existence using `contains_key()`
- Provides detailed error messages with full JSON dump on failure

---

#### `the JSON field "<field_path>" should have "<sub_field>"`

**Purpose:** Verifies that a nested JSON field contains a specific sub-field. Enables validation of nested object structures.

**Location:** `../../crates/acceptance/src/steps/xtask_devex.rs` (lines 2871-2904)

**Usage Example:**

```gherkin
When I run "cargo xtask idp-snapshot"
Then the output should be valid JSON
And the JSON field "governance_health" should have "status"
And the JSON field "governance_health" should have "ac_coverage"
```

**Implementation Details:**
- Extracts JSON from command output
- Navigates to the specified field path (e.g., "governance_health")
- Validates that the field is an object (not string, array, etc.)
- Checks for sub-field existence
- Provides context-aware error messages showing the actual field value

**Use Cases:**
- Validating nested API response structures
- Checking governance health sub-fields
- Verifying task hint metadata structures

---

#### `the file should contain valid JSON`

**Purpose:** Verifies that a file written by a command contains valid, parseable JSON.

**Location:** `../../crates/acceptance/src/steps/xtask_devex.rs` (lines 2906-2940)

**Usage Example:**

```gherkin
When I run "cargo xtask idp-snapshot --output /tmp/idp-test.json"
Then the command should succeed
And the file "/tmp/idp-test.json" should exist
And the file should contain valid JSON
```

**Implementation Details:**
- Attempts to locate file path from:
  1. `test_evidence_path` in xtask context (if available)
  2. Command output (looks for `/tmp/` paths)
- Reads file content
- Parses as JSON to validate structure
- Provides detailed error with parse error and file content on failure

**Use Cases:**
- Validating JSON output files from `idp-snapshot`
- Checking SBOM JSON generation
- Verifying release bundle JSON artifacts

**Known Limitations:**
- Currently expects file path to be in `/tmp/` or set in `test_evidence_path`
- May need extension for other file path patterns

---

### 2. Environment Variable Steps

#### `the environment variable "<var_name>" is set to "<var_value>"`

**Purpose:** Sets an environment variable for subsequent command executions within the same scenario.

**Location:** `../../crates/acceptance/src/steps/xtask_devex.rs` (lines 209-212)

**Usage Example:**

```gherkin
Given the environment variable "CI" is set to "1"
And the environment variable "XTASK_NONINTERACTIVE" is set to "1"
When I run "cargo xtask selftest"
Then the command should succeed
And the output should indicate CI mode
```

**Implementation Details:**
- Stores environment variable in `world.xtask_context_mut().env`
- Variables are passed to child processes via `cmd.env()`
- Scoped to the current test scenario (isolated per World instance)

**Use Cases:**
- Testing CI-specific behavior
- Enabling low-resource mode (`XTASK_LOW_RESOURCES=1`)
- Setting non-interactive mode for automation
- Testing environment-dependent command behavior

**Related Steps:**
- `XTASK_LOW_RESOURCES is set to "<value>"` (specialized step for low-resource mode)
- `XTASK_LOW_RESOURCES is not set` (unsets the variable)

---

## Test Results

### Compilation

✅ **All code compiles successfully**

```
cargo check --package acceptance
Finished `dev` profile [unoptimized + debuginfo] target(s) in 47.18s
```

### BDD Test Suite

✅ **All acceptance tests pass**

```
cargo xtask bdd
✓ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

### Step Usage in Features

The new steps are actively used in the following feature files:

**JSON validation steps:**
- `specs/features/xtask_devex.feature` (idp-snapshot scenarios)
- `specs/features/agent_hints.feature` (CLI JSON output)
- `specs/features/platform_introspection.feature` (API responses)
- `specs/features/platform_tasks.feature` (task filtering)
- `specs/features/forks.feature` (fork-list JSON output)
- `specs/features/questions.feature` (questions-list JSON output)
- `specs/features/friction.feature` (friction-list JSON output)

**Environment variable steps:**
- `specs/features/git_hooks.feature` (pre-commit with XTASK_LOW_RESOURCES)
- `specs/features/xtask_devex.feature` (CI mode testing)

---

## Implementation Patterns

### JSON Extraction Helper

All JSON validation steps use the `extract_json_from_output()` helper function to handle:
- Nix devshell messages ("DevShell ready — try: just checks")
- Cargo build messages ("Finished `dev` profile...")
- Multi-line output with JSON embedded

This ensures robust JSON parsing regardless of surrounding output noise.

### Error Messages

All steps provide detailed, actionable error messages:
- Show expected vs. actual values
- Include full JSON structure on field validation failures
- Display file content on parse errors
- Provide context (field path, sub-field name) in nested checks

### Test Isolation

Environment variables are:
- Scoped per `World` instance (one per scenario)
- Stored in `xtask_context.env` HashMap
- Passed explicitly to child processes via `cmd.env()`
- Not leaked between scenarios

---

## Related Work

### Batch 1 Steps (Already Implemented)

- HTTP request/response steps
- Basic command execution steps
- File existence and content checks
- Basic cleanup steps
- **See:** `docs/receipts/BDD_IMPLEMENTATION_BATCH1.md`

### Future Batches (Potential)

- Array field validation (check array contents, length constraints)
- Numeric field validation (ranges, comparisons)
- Date/timestamp validation
- JSON schema validation against OpenAPI specs
- Performance assertion steps (execution time, memory usage)

---

## Maintenance Notes

### Adding New JSON Validation Steps

When adding new JSON validation steps:

1. **Use consistent patterns:**
   - Extract JSON with `extract_json_from_output()`
   - Provide detailed error messages with context
   - Handle both CLI output and HTTP response JSON

2. **Consider both contexts:**
   - CLI commands store JSON in `world.cli_json_output`
   - HTTP responses store JSON in `world.last_response.body`
   - Some steps (like agent_hints.rs) support both

3. **Document step usage:**
   - Add examples to `docs/testing/BDD_STEP_LIBRARY.md`
   - Link to feature files that use the step
   - Explain error message format

### Testing New Steps

Before committing new steps:
1. Run `cargo check --package acceptance` (compilation)
2. Run `cargo xtask bdd` (full test suite)
3. Run specific feature with `cargo test --package acceptance --test acceptance -- "<scenario name>"`
4. Verify error messages are helpful (intentionally fail the step)

---

## References

- **Cucumber Documentation:** <https://cucumber.io/docs/cucumber/>
- **Cucumber Rust:** <https://github.com/cucumber-rs/cucumber>
- **Step Library Reference:** `../testing/BDD_STEP_LIBRARY.md`
- **Batch 1 Implementation:** `BDD_IMPLEMENTATION_BATCH1.md`
- **Acceptance Test Crate:** `../../crates/acceptance/`
- **Feature Files:** `../../specs/features/`
- **World State:** `../../crates/acceptance/src/world.rs`

---

## Summary

Batch 2 implementation successfully adds:

✅ **3 new JSON validation steps:**
- `the JSON should contain field "<field_name>"`
- `the JSON field "<field_path>" should have "<sub_field>"`
- `the file should contain valid JSON`

✅ **1 new environment variable step:**
- `the environment variable "<var_name>" is set to "<var_value>"`

✅ **All tests passing**

These steps enhance the BDD framework's ability to validate complex JSON structures, test environment-dependent behavior, and verify file-based outputs. They are actively used across multiple feature files and provide clear, actionable error messages for test failures.

---

**Status:** ✅ Ready for production use

**Maintainer:** Claude (Autonomous Agent)
**Date:** 2025-12-02
**Template Version:** 3.3.6
