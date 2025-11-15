# Policy Directory

This directory contains Rego policies for validating various configuration files in the project.

## Overview

Policies are implemented using [Open Policy Agent](https://www.openpolicyagent.org/) and tested with [Conftest](https://www.conftest.dev/).

## Available Policies

### Core Policies

- **ledger.rego** - Validates spec_ledger.yaml structure and AC mappings
- **features.rego** - Validates feature flag definitions
- **flags.rego** - Validates feature flag integrity
- **privacy.rego** - Validates PII field annotations
- **template_core.rego** - Ensures template foundation ACs are not removed
- **llm.rego** - Validates LLM contextpack configuration

### Policy Details

#### LLM Policy (`llm.rego`)

Validates `.llm/contextpack.yaml` to ensure:

- **Required tasks exist**: `implement_ac`, `implement_feature`, `debug_tests`
- **Valid max_bytes**: Every task has `max_bytes > 0`
- **Non-empty includes**: Every task has at least one include pattern
- **No unknown fields**: Only `max_bytes`, `include`, `description` are allowed

This policy protects against misconfiguration of the LLM context bundling system.

## Running Tests

```bash
# Run all policy tests
cargo xtask policy-test

# Requires conftest to be installed
# Via nix: nix develop
# Via homebrew: brew install conftest
```

## Test Fixtures

Test fixtures are located in `policy/testdata/` and follow the naming convention:

- `{area}_valid.json` - Valid configuration (should pass)
- `{area}_{error_type}.json` - Invalid configurations (should fail)

Example fixtures for llm policy:
- `llm_valid.json` - Valid contextpack structure
- `llm_missing_include.json` - Task with empty include list
- `llm_zero_bytes.json` - Task with max_bytes = 0
- `llm_missing_required_task.json` - Missing required task

## Writing New Policies

1. Create `policy/{area}.rego` with validation rules
2. Create test fixtures in `policy/testdata/`
3. Add the area to `POLICY_AREAS` in `crates/xtask/src/commands/policy_test.rs`
4. Run `cargo xtask policy-test` to verify

## Policy Structure

Each Rego policy should:

```rego
package {area}

# Validation rules
deny[msg] {
    # conditions that should fail validation
    msg := sprintf("Error message: %s", [variable])
}

# Helper functions
helper_function(arg) {
    # validation logic
}
```

## CI Integration

Policies are run as part of:
- `cargo xtask check` - Runs all validation checks
- `cargo xtask selftest` - Full template test suite
- CI pipeline before deployment
