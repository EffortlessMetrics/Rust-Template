<!-- doclint:disable orphan-version -->
# BDD Step Library Reference

**Version:** 3.3.8
**Last Updated:** 2025-12-02

This document provides a comprehensive reference for all reusable BDD step definitions available in the acceptance test suite. These steps can be used across all feature files to create readable, maintainable test scenarios.

---

## Table of Contents

1. [File Operations](#file-operations)
2. [Command Execution](#command-execution)
3. [JSON Assertions](#json-assertions)
4. [String Assertions](#string-assertions)
5. [HTTP Operations](#http-operations)
6. [Environment Variables](#environment-variables)
7. [Best Practices](#best-practices)

---

## Overview

This step library is organized into modules for maintainability:

- **`common.rs`** - File operations, string assertions, JSON validation (Batch 1)
- **`xtask_devex.rs`** - XTask command execution, environment variables, JSON validation (Batch 2)
- **`template_core.rs`** - HTTP request/response handling
- **`agent_hints.rs`** - Agent hints API validation
- **`platform_ui.rs`** - Platform UI interactions
- **`governance_tasks.rs`** - Governance workflows
- **`config_validation.rs`** - Configuration validation

**Implementation Batches:**
- **Batch 1** (2025-12-02): 24 steps - File operations, basic assertions
- **Batch 2** (2025-12-02): 4 steps - JSON validation, environment variables

---

## File Operations

### File Existence Checks

#### `Then the file "{path}" should exist`

Asserts that a file exists at the given path (relative to workspace root).

**Example:**

```gherkin
Then the file "README.md" should exist
Then the file "docs/adr/ADR-001.md" should exist
```

#### `Then the file "{path}" should not exist`

Asserts that a file does NOT exist at the given path.

**Example:**

```gherkin
Then the file "temp/cache.tmp" should not exist
```

#### `Then the directory "{path}" should exist`

Asserts that a directory exists at the given path.

**Example:**

```gherkin
Then the directory ".git/hooks" should exist
Then the directory "specs/features" should exist
```

#### `Then the directory "{path}" should not exist`

Asserts that a directory does NOT exist at the given path.

**Example:**

```gherkin
Then the directory "temp/cache" should not exist
```

---

### File Content Checks

#### `Then the file "{path}" should contain "{text}"`

Asserts that a file contains the specified text (substring match).

**Example:**

```gherkin
Then the file "README.md" should contain "Rust-as-Spec"
Then the file "Cargo.toml" should contain "[workspace]"
```

#### `Then the file "{path}" should not contain "{text}"`

Asserts that a file does NOT contain the specified text.

**Example:**

```gherkin
Then the file "CHANGELOG.md" should not contain "UNRELEASED"
```

#### `Then the file "{path}" should match pattern "{regex}"`

Asserts that file content matches the given regular expression pattern.

**Example:**

```gherkin
Then the file "version.txt" should match pattern "^\d+\.\d+\.\d+$"
Then the file "log.txt" should match pattern "ERROR.*authentication"
```

#### `Then the file "{path}" should be empty`

Asserts that a file exists but has no content (or only whitespace).

**Example:**

```gherkin
Then the file "output.log" should be empty
```

#### `Then the file "{path}" should not be empty`

Asserts that a file has content.

**Example:**

```gherkin
Then the file "report.json" should not be empty
```

---

### File Permissions (Unix only)

#### `Then the file "{path}" should be executable`

Asserts that a file has executable permissions (Unix-only).

**Example:**

```gherkin
Then the file ".git/hooks/pre-commit" should be executable
```

#### `Then the file "{path}" should not be executable`

Asserts that a file does NOT have executable permissions (Unix-only).

**Example:**

```gherkin
Then the file "README.md" should not be executable
```

---

### File Creation and Manipulation

#### `Given a file "{path}" with content:`

Creates a file with the specified content (multiline supported via docstring).

**Example:**

```gherkin
Given a file "test.yaml" with content:
  """
  version: 1.0
  name: test
  """
```

#### `Given a file "{path}" exists`

Ensures a file exists (creates empty file if it doesn't).

**Example:**

```gherkin
Given a file "temp/marker.txt" exists
```

#### `Given a directory "{path}" exists`

Ensures a directory exists (creates it if necessary).

**Example:**

```gherkin
Given a directory "temp/cache" exists
```

#### `When I delete the file "{path}"`

Deletes a file if it exists.

**Example:**

```gherkin
When I delete the file "temp/old-cache.json"
```

#### `When I delete the directory "{path}"`

Deletes a directory and all its contents.

**Example:**

```gherkin
When I delete the directory "temp/cache"
```

#### `When I create a file "{path}" with content "{text}"`

Creates a file with inline content (single line).

**Example:**

```gherkin
When I create a file "version.txt" with content "3.3.6"
```

---

## Command Execution

### Exit Code Assertions

#### `Then the exit code should be {code}`

Asserts that the last command exited with the specified code.

**Example:**

```gherkin
When I run "cargo xtask check"
Then the exit code should be 0
```

#### `Then the exit code should not be {code}`

Asserts that the last command did NOT exit with the specified code.

**Example:**

```gherkin
When I run "cargo xtask invalid-command"
Then the exit code should not be 0
```

---

## JSON Assertions

### Field Presence (Batch 1)

#### `Then the JSON output should have field "{field}"`

Asserts that the parsed JSON output contains the specified top-level field.

**Example:**

```gherkin
When I run "cargo xtask status --format json"
Then the JSON output should have field "governance"
Then the JSON output should have field "health"
```

#### `Then the JSON output should not have field "{field}"`

Asserts that the parsed JSON output does NOT contain the specified field.

**Example:**

```gherkin
Then the JSON output should not have field "internal_debug"
```

---

### Field Value Assertions (Batch 1)

#### `Then the JSON field "{field}" should equal "{value}"`

Asserts that a JSON field equals the specified string value.

**Example:**

```gherkin
Then the JSON field "status" should equal "ok"
Then the JSON field "version" should equal "3.3.6"
```

#### `Then the JSON field "{field}" should contain "{substring}"`

Asserts that a JSON field contains the specified substring.

**Example:**

```gherkin
Then the JSON field "message" should contain "Success"
```

---

### Advanced JSON Validation (Batch 2)

#### `Then the JSON should contain field "{field_name}"`

**Location:** `xtask_devex.rs` (lines 2850-2869)

Verifies that JSON output from a CLI command contains a specific top-level field. Automatically extracts JSON from command output, skipping Nix/cargo messages.

**Example:**

```gherkin
When I run "cargo xtask idp-snapshot"
Then the output should be valid JSON
And the JSON should contain field "timestamp"
And the JSON should contain field "template_version"
And the JSON should contain field "governance_health"
```

#### `Then the JSON field "{field_path}" should have "{sub_field}"`

**Location:** `xtask_devex.rs` (lines 2871-2904)

Verifies that a nested JSON field contains a specific sub-field. Enables validation of nested object structures.

**Example:**

```gherkin
When I run "cargo xtask idp-snapshot"
Then the output should be valid JSON
And the JSON field "governance_health" should have "status"
And the JSON field "governance_health" should have "ac_coverage"
```

#### `Then the file should contain valid JSON`

**Location:** `xtask_devex.rs` (lines 2906-2940)

Verifies that a file written by a command contains valid, parseable JSON.

**Example:**

```gherkin
When I run "cargo xtask idp-snapshot --output /tmp/idp-test.json"
Then the command should succeed
And the file "/tmp/idp-test.json" should exist
And the file should contain valid JSON
```

---

## String Assertions

### Output Content Checks

#### `Then the output should contain "{text}"`

Asserts that command output contains the specified text.

**Example:**

```gherkin
When I run "cargo xtask help"
Then the output should contain "Available commands"
```

#### `Then the output should not contain "{text}"`

Asserts that command output does NOT contain the specified text.

**Example:**

```gherkin
Then the output should not contain "ERROR"
Then the output should not contain "FAILED"
```

#### `Then the output should match pattern "{regex}"`

Asserts that output matches the given regular expression.

**Example:**

```gherkin
Then the output should match pattern "Version: \d+\.\d+\.\d+"
```

#### `Then the output should be empty`

Asserts that command output is empty (or only whitespace).

**Example:**

```gherkin
Then the output should be empty
```

#### `Then the output should not be empty`

Asserts that command produced some output.

**Example:**

```gherkin
When I run "cargo xtask ac-status"
Then the output should not be empty
```

---

## HTTP Operations

### HTTP Request Steps

#### `When I GET {endpoint}`

Sends a GET request to the specified endpoint.

**Example:**

```gherkin
When I GET /health
When I GET /version
```

#### `When I send a GET request to "{endpoint}"`

Alternative syntax for GET requests (more explicit).

**Example:**

```gherkin
When I send a GET request to "/platform/status"
```

---

### HTTP Response Assertions

#### `Then I receive {status_code} with status "{status_value}"`

Asserts HTTP status code and JSON status field.

**Example:**

```gherkin
When I GET /health
Then I receive 200 with status "ok"
```

#### `Then the response status code should be {code}`

Asserts HTTP status code.

**Example:**

```gherkin
Then the response status code should be 200
```

#### `Then I receive a {digit}xx response`

Asserts HTTP status code range (e.g., 2xx, 4xx, 5xx).

**Example:**

```gherkin
Then I receive a 2xx response
Then I receive a 4xx response
```

---

### HTTP Header Assertions

#### `Then the response includes "{header}" header`

Asserts that a response header is present.

**Example:**

```gherkin
Then the response includes "X-Request-ID" header
Then the response includes "Content-Type" header
```

#### `Then the response includes "{header}" header with value "{value}"`

Asserts a response header has a specific value.

**Example:**

```gherkin
Then the response includes "Content-Type" header with value "application/json"
```

---

### HTTP Body Assertions

#### `Then the response body contains "{field}" field`

Asserts that JSON response body has a field.

**Example:**

```gherkin
Then the response body contains "error" field
Then the response body contains "requestId" field
```

#### `Then the response body contains "{text}"`

Asserts that response body (raw text) contains text.

**Example:**

```gherkin
Then the response body contains "Success"
```

---

## Environment Variables

### Setting Environment Variables (Batch 2)

#### `Given the environment variable "{var_name}" is set to "{var_value}"`

**Location:** `xtask_devex.rs` (lines 209-212)

Sets an environment variable for subsequent command executions within the same scenario. Variables are scoped per test scenario and do not leak between tests.

**Example:**

```gherkin
Given the environment variable "CI" is set to "1"
And the environment variable "XTASK_NONINTERACTIVE" is set to "1"
When I run "cargo xtask selftest"
Then the command should succeed
And the output should indicate CI mode
```

**Use Cases:**
- Testing CI-specific behavior
- Enabling low-resource mode (`XTASK_LOW_RESOURCES=1`)
- Setting non-interactive mode for automation
- Testing environment-dependent command behavior

---

## Best Practices

### 1. Use Descriptive File Paths

Always use clear, descriptive file paths that indicate what the file is for:

✅ **Good:**

```gherkin
Then the file "docs/adr/ADR-001-auth-strategy.md" should exist
```

❌ **Bad:**

```gherkin
Then the file "file1.md" should exist
```

---

### 2. Prefer Specific Assertions

Use the most specific assertion available:

✅ **Good:**

```gherkin
Then the exit code should be 0
Then the file "README.md" should contain "Quick Start"
```

❌ **Bad:**

```gherkin
Then the command should succeed  # Less specific
Then the output should contain "README"  # Could match unrelated text
```

---

### 3. Chain Steps Logically

Organize steps in a clear Given-When-Then structure:

✅ **Good:**

```gherkin
Given a file "config.yaml" exists
When I run "cargo xtask validate-config"
Then the exit code should be 0
And the output should contain "Config valid"
```

❌ **Bad:**

```gherkin
When I run "cargo xtask validate-config"
Given a file "config.yaml" exists  # Given should come first
```

---

### 4. Use Regex Patterns for Flexible Matching

When exact matches are too brittle, use regex patterns:

✅ **Good:**

```gherkin
Then the file "CHANGELOG.md" should match pattern "## \[\d+\.\d+\.\d+\]"
```

---

### 5. Clean Up Test Artifacts

Always clean up files/directories created during tests:

✅ **Good:**

```gherkin
Scenario: Generate report
  Given a directory "temp/reports" exists
  When I run "cargo xtask generate-report"
  Then the file "temp/reports/output.json" should exist
  And I delete the directory "temp/reports"  # Cleanup
```

---

### 6. Avoid Hardcoded Paths in Steps

Use workspace-relative paths, not absolute paths:

✅ **Good:**

```gherkin
Then the file "docs/README.md" should exist
```

❌ **Bad:**

```gherkin
Then the file "/home/user/project/docs/README.md" should exist
```

---

## Step Definition Locations

All step definitions are organized by category in:

- **Common operations**: `crates/acceptance/src/steps/common.rs` (Batch 1 - 24 steps)
- **XTask devex**: `crates/acceptance/src/steps/xtask_devex.rs` (Batch 2 - 4 steps)
- **Template core**: `crates/acceptance/src/steps/template_core.rs`
- **Platform UI**: `crates/acceptance/src/steps/platform_ui.rs`
- **Agent hints**: `crates/acceptance/src/steps/agent_hints.rs`
- **Governance**: `crates/acceptance/src/steps/governance_tasks.rs`
- **Config validation**: `crates/acceptance/src/steps/config_validation.rs`

---

## Adding New Steps

When you need a new step definition:

1. **Check if it already exists** in this document or the step definition files
2. **Determine the category** (file ops, HTTP, JSON, etc.)
3. **Add the step** to the appropriate module (`common.rs` for generic steps)
4. **Use regex patterns** for flexibility where appropriate
5. **Update this documentation** with the new step
6. **Test the step** with at least one scenario

---

## Implementation Summary

### Batch 1 (2025-12-02): File Operations & Basic Assertions

- **24 new steps** implemented in `common.rs`
- File existence, content checks, permissions
- String and JSON assertions
- File manipulation (create, delete)
- Full documentation in `BDD_IMPLEMENTATION_SUMMARY.md`

### Batch 2 (2025-12-02): JSON Validation & Environment Variables

- **4 new steps** implemented in `xtask_devex.rs`
- Advanced JSON field validation (nested fields)
- JSON file validation
- Environment variable management
- Full documentation in `cucumber_steps_batch2.md`

**Total**: 28 new reusable step definitions across 2 batches

---

## Examples of Complete Scenarios

### Example 1: File Creation and Validation

```gherkin
Scenario: Create and validate configuration file
  Given a directory "config" exists
  When I create a file "config/app.yaml" with content "version: 1.0"
  Then the file "config/app.yaml" should exist
  And the file "config/app.yaml" should contain "version: 1.0"
  And the file "config/app.yaml" should not be empty
```

### Example 2: Command Execution with Output Validation

```gherkin
Scenario: Run health check command
  When I run "cargo xtask doctor"
  Then the exit code should be 0
  And the output should contain "Environment check"
  And the output should not contain "ERROR"
  And the output should match pattern "✓.*passed"
```

### Example 3: HTTP API Testing

```gherkin
Scenario: Platform status endpoint
  When I send a GET request to "/platform/status"
  Then the response status code should be 200
  And the response body contains "governance" field
  And the response includes "Content-Type" header
```

### Example 4: JSON Output Validation

```gherkin
Scenario: Export task list as JSON
  When I run "cargo xtask tasks-list --format json"
  Then the exit code should be 0
  And the JSON output should have field "tasks"
  And the JSON field "status" should equal "ok"
```

### Example 5: Environment Variables and CI Mode

```gherkin
Scenario: Test CI mode behavior
  Given the environment variable "CI" is set to "1"
  And the environment variable "XTASK_NONINTERACTIVE" is set to "1"
  When I run "cargo xtask selftest"
  Then the exit code should be 0
  And the output should not contain "interactive prompt"
```

---

## Troubleshooting

### Step Not Found

If you get an "undefined step" error:

1. Check this document for the exact step syntax
2. Verify the step is imported in `crates/acceptance/src/steps/mod.rs`
3. Rebuild the acceptance crate: `cargo build -p acceptance`

### Step Assertion Fails

If a step assertion fails:

1. Check the error message for expected vs actual values
2. Use `And the output should contain "DEBUG"` to inspect command output
3. Add `--verbose` flags to commands for more detailed output
4. Check file paths are relative to workspace root

### Path Resolution Issues

All file paths are resolved relative to the workspace root. To debug:

```gherkin
When I run "pwd"
Then the output should not be empty  # Shows current directory
```

---

## Version History

- **v3.3.6** (2025-12-02): Comprehensive step library with 28 reusable steps
  - Batch 1: 24 steps (file ops, basic assertions)
  - Batch 2: 4 steps (JSON validation, environment variables)
- Future versions will add more specialized steps as needed

---

## References

- **Cucumber Documentation:** <https://cucumber.io/docs/cucumber/>
- **Cucumber Rust:** <https://github.com/cucumber-rs/cucumber>
- **Acceptance Test Crate:** `../../crates/acceptance/`
- **Feature Files:** `../../specs/features/`
- **World State:** `../../crates/acceptance/src/world.rs`
- **Implementation Summary (Batch 1):** `../receipts/BDD_IMPLEMENTATION_BATCH1.md`
- **Implementation Summary (Batch 2):** `../receipts/BDD_IMPLEMENTATION_BATCH2.md`

---

**Maintainer Note:** Keep this document synchronized with step definitions in `crates/acceptance/src/steps/`. This is the single source of truth for BDD step usage and examples.
