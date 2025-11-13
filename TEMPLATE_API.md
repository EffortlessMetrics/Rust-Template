# Template API Reference

This document defines the stable interfaces and contracts provided by the Rust Template.
These APIs are considered stable and breaking changes will be versioned as major releases.

## xtask CLI Commands

The `xtask` binary is the single entrypoint for all development and CI operations.

### `xtask check`

Run all code quality checks: formatting, linting, and tests.

**Usage:**
```bash
cargo run -p xtask -- check
# or in Nix shell:
nix develop -c cargo run -p xtask -- check
```

**Behavior:**
1. Runs `cargo fmt --all -- --check` (formatting verification)
2. Runs `cargo clippy --all-targets --all-features -- -D warnings` (lint checks)
3. Runs `cargo test --workspace --exclude acceptance` (unit/integration tests)

**Exit codes:**
- `0`: All checks passed
- Non-zero: One or more checks failed

**Notes:**
- Stops at first failure
- Exclusion of `acceptance` crate is intentional (use `xtask bdd` for those)

---

### `xtask bdd`

Run BDD acceptance tests and generate JUnit XML output.

**Usage:**
```bash
cargo run -p xtask -- bdd
```

**Behavior:**
1. Runs `cargo test -p acceptance --test acceptance`
2. Generates JUnit XML at `target/junit/acceptance.xml`

**Exit codes:**
- `0`: All scenarios passed
- Non-zero: One or more scenarios failed

**Output artifacts:**
- `target/junit/acceptance.xml`: JUnit test results

---

### `xtask bundle <task>`

Generate an LLM context bundle for a specific task.

**Usage:**
```bash
cargo run -p xtask -- bundle implement_ac
cargo run -p xtask -- bundle implement_feature
cargo run -p xtask -- bundle debug_tests
```

**Parameters:**
- `<task>`: Task name as defined in `.llm/contextpack.yaml`

**Behavior:**
1. Invokes `scripts/make-context.sh <task>`
2. Reads task configuration from `.llm/contextpack.yaml`
3. Collects files matching `include` patterns
4. Generates markdown bundle at `.llm/bundle/<task>.md`

**Exit codes:**
- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

**Output artifacts:**
- `.llm/bundle/<task>.md`: Generated context bundle

---

## Policy Input Schemas

All policies are written in Rego and tested via `conftest`. Each policy expects JSON input in a specific schema.

### Ledger Policy (`policy/ledger.rego`)

**Purpose:** Ensure every AC has at least one mapped test.

**Input schema:**
```json
{
  "stories": [
    {
      "id": "US-###",
      "requirements": [
        {
          "id": "REQ-###",
          "acceptance_criteria": [
            {
              "id": "AC-###",
              "text": "...",
              "tests": [
                { "type": "bdd", "tag": "@AC-###" }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

**Rules:**
- Denies if `tests` array is missing or empty
- Each AC must have at least one test reference

**Usage in CI:**
```bash
yq -o=json specs/spec_ledger.yaml | conftest test -p policy/ledger.rego -
```

---

### Features Policy (`policy/features.rego`)

**Purpose:** Ensure features only reference ACs that exist in the ledger.

**Input schema:**
```json
{
  "features": [
    {
      "id": "FT-###",
      "acceptance_criteria": ["AC-###", "AC-###"]
    }
  ],
  "ledger_ac_ids": ["AC-###", "AC-###", ...]
}
```

**Rules:**
- Denies if a feature references an AC ID not in `ledger_ac_ids`

**Usage in CI:**
```bash
# Extract AC IDs from ledger
yq -o=json specs/spec_ledger.yaml | \
  jq '[.stories[].requirements[].acceptance_criteria[].id]' > /tmp/ledger_ac_ids.json

# Extract features
yq -o=json features/*.yaml | \
  jq -s '[.[] | {id, acceptance_criteria: (.acceptance_criteria // [])}]' > /tmp/features.json

# Combine and test
jq -s '{features:.[0], ledger_ac_ids:.[1]}' \
  /tmp/features.json /tmp/ledger_ac_ids.json | \
  conftest test -p policy/features.rego -
```

---

### Flags Policy (`policy/flags.rego`)

**Purpose:** Validate flag ownership, rollouts, and percentages.

**Input schema:**
```json
{
  "flags": [
    {
      "key": "flag_name",
      "owner": "team-name",
      "default": false,
      "expires_at": "2026-12-31"
    }
  ],
  "rollouts": {
    "dev": { "flag_name": 100 },
    "staging": { "flag_name": 50 },
    "prod": { "flag_name": 0 }
  }
}
```

**Rules:**
- Denies if flag has no `owner` or empty owner
- Denies if rollout percentage < 0 or > 100
- Denies if rollout references non-existent flag

**Usage in CI:**
```bash
yq -o=json flags/registry.yaml > /tmp/flags.json
yq -o=json flags/rollouts.yaml | jq '{rollouts: .environments}' > /tmp/rollouts.json
jq -s '.[0] * .[1]' /tmp/flags.json /tmp/rollouts.json | \
  conftest test -p policy/flags.rego -
```

---

### Privacy Policy (`policy/privacy.rego`)

**Purpose:** Ensure PII fields have owners and valid retention periods.

**Input schema:**
```json
{
  "fields": [
    {
      "path": "user.email",
      "classification": "PII",
      "owner": "team-identity",
      "retention": "365d",
      "purpose": "..."
    }
  ]
}
```

**Rules:**
- Denies if PII field has no `owner` or empty owner
- Denies if PII field has no `retention`
- Denies if retention format doesn't match `^\d+[dwmy]$` (e.g., "365d", "2y")

**Field is considered PII if:**
- `classification == "PII"`, OR
- `pii` field is truthy

**Usage in CI:**
```bash
yq -o=json specs/privacy.yaml | conftest test -p policy/privacy.rego -
```

---

## AC Status Script (`scripts/ac_status.py`)

**Purpose:** Map test results to ACs and generate status report.

**Inputs:**
1. `specs/spec_ledger.yaml`: AC definitions
2. `specs/features/*.feature`: Gherkin scenarios with `@AC-####` tags
3. `target/junit/acceptance.xml`: JUnit test results

**Output:**
- `docs/feature_status.md`: Markdown table with AC statuses

**Behavior:**
1. Extracts all AC IDs from ledger
2. Maps scenarios to ACs via `@AC-####` tags
3. Maps testcases to scenarios (normalizing names)
4. Determines AC status:
   - `✅ pass`: All mapped tests passed
   - `❌ fail`: Any mapped test failed
   - `❓ unknown`: No mapped tests
5. Reports unmapped ACs and scenarios

**Exit codes:**
- `0`: All mapped ACs passed
- Non-zero: One or more ACs failed

**Usage:**
```bash
python3 scripts/ac_status.py
```

---

## LLM Context Bundler (`scripts/make-context.sh`)

**Purpose:** Generate focused context bundles for LLM consumption.

**Configuration:** `.llm/contextpack.yaml`

**Schema:**
```yaml
tasks:
  task_name:
    max_bytes: 250000
    include:
      - "glob/pattern/*.yaml"
      - "another/pattern/**/*.rs"
    description: "Optional description"
```

**Behavior:**
1. Reads task configuration from `.llm/contextpack.yaml`
2. Resolves `include` globs via `git ls-files`
3. Filters files via `.llm/.llmignore` (if present)
4. Concatenates files with `# FILE: path` headers
5. Enforces `max_bytes` limit
6. Writes to `.llm/bundle/<task>.md`

**Output format:**
```markdown
# Context Bundle: task_name

Generated from commit: <sha>

## Purpose
...

---

# FILE: path/to/file

<file contents>

---

# FILE: another/file

<file contents>
```

**Exit codes:**
- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

**Usage:**
```bash
bash scripts/make-context.sh <task>
```

---

## Test Scripts

### `scripts/test-policies.sh`

Tests all Rego policies against fixtures in `policy/testdata/`.

**Usage:**
```bash
bash scripts/test-policies.sh
```

**Exit codes:**
- `0`: All policy tests passed
- Non-zero: One or more policy tests failed

---

### `scripts/test-ac-status.sh`

Tests the AC status script.

**Usage:**
```bash
bash scripts/test-ac-status.sh
```

**Exit codes:**
- `0`: All AC status tests passed
- Non-zero: One or more tests failed

---

### `scripts/test-bundler.sh`

Tests the LLM context bundler.

**Usage:**
```bash
bash scripts/test-bundler.sh
```

**Exit codes:**
- `0`: All bundler tests passed
- Non-zero: One or more tests failed

---

### `scripts/test-all.sh`

Runs all test scripts in sequence.

**Usage:**
```bash
bash scripts/test-all.sh
```

**Exit codes:**
- `0`: All test suites passed
- Non-zero: One or more test suites failed

---

## Versioning Policy

This template follows semantic versioning:

- **Major version (X.0.0)**: Breaking changes to any API defined in this document
- **Minor version (0.X.0)**: New features, backward-compatible changes
- **Patch version (0.0.X)**: Bug fixes, documentation updates

**Examples of breaking changes:**
- Changing xtask command names or arguments
- Changing policy input schemas
- Changing AC status script expected inputs/outputs
- Changing contextpack.yaml schema

**Examples of non-breaking changes:**
- Adding new xtask commands
- Adding new policy rules (more strict)
- Adding new fields to JSON schemas (if policies ignore unknown fields)
- Improving error messages
