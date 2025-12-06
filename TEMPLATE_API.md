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
1. Loads task configuration from `.llm/contextpack.yaml`
2. Uses `git ls-files` to find files matching `include` patterns
3. Applies `.llm/.llmignore` exclusion patterns
4. Enforces `max_bytes` size limit per task
5. Generates markdown bundle at `.llm/bundle/<task>.md`

**Exit codes:**
- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

**Output artifacts:**
- `.llm/bundle/<task>.md`: Generated context bundle

---

### `xtask ac-status`

Generate AC status report from acceptance test results.

**Usage:**
```bash
cargo run -p xtask -- ac-status
# or in Nix shell:
nix develop -c cargo run -p xtask -- ac-status
```

**Behavior:**
1. Reads `specs/spec_ledger.yaml` to extract all AC definitions
2. **Primary:** Parses AC coverage JSONL (`target/ac/coverage.jsonl`) from BDD harness
3. **Fallbacks:** JSON report or JUnit XML + feature file parsing (legacy)
4. Maps testcases → scenarios → ACs
5. Computes status for each AC (pass/fail/unknown)
6. Generates `docs/feature_status.md` with status table

See `docs/design/ac-coverage-format.md` for coverage.jsonl specification.

**Exit codes:**
- `0`: All ACs passed or unknown
- Non-zero: One or more ACs failed

**Output artifacts:**
- `docs/feature_status.md`: AC status table with pass/fail/unknown indicators

**Status logic:**
- **Pass (✅)**: All mapped testcases passed
- **Fail (❌)**: Any mapped testcase failed
- **Unknown (❓)**: AC has no mapped scenarios or testcases

**Notes:**
- Normalizes testcase names by removing ` (row N)` and ` (example N)` suffixes
- Reports unmapped ACs (no scenarios) and unmapped scenarios (invalid AC refs)
- Used by `xtask selftest` and `ci-ac.yml` workflow

---

### `xtask policy-test`

Test Rego policies with conftest.

**Usage:**
```bash
cargo run -p xtask -- policy-test
```

**Behavior:**
1. Checks if conftest is available on PATH
2. Runs policy tests for each policy area (ledger, features, flags, privacy)
3. Tests against fixtures in `policy/testdata/`
4. Validates that valid fixtures pass and invalid fixtures fail
5. Reports comprehensive test results

**Exit codes:**
- `0`: All policy tests passed
- Non-zero: One or more policy tests failed or conftest not available

**Policy areas tested:**
- **Ledger** (`policy/ledger.rego`) - Ensures every AC has tests
- **Features** (`policy/features.rego`) - Validates feature-AC references
- **Flags** (`policy/flags.rego`) - Validates flag ownership and rollouts
- **Privacy** (`policy/privacy.rego`) - Ensures PII fields have owners and retention

**Fixtures:**
Each policy area has test fixtures in `policy/testdata/`:
- `{area}_valid.json` - Should pass policy checks
- `{area}_invalid.json` - Should fail policy checks
- `{area}_missing_tests.json` - Should fail (for ledger)
- `{area}_unknown_ac.json` - Should fail (for features)

**Prerequisites:**
Requires `conftest` on PATH. Available in Nix shell or install separately:
- macOS: `brew install conftest`
- Linux: See https://www.conftest.dev/install/
- Nix: `nix develop` (automatically available)

**Notes:**
- Used by `xtask selftest` but gracefully degrades if conftest unavailable
- Each policy area is tested independently
- Provides clear pass/fail output for each fixture

---

### `xtask selftest`

Run the complete template self-test suite (used in CI).

**Usage:**
```bash
cargo run -p xtask -- selftest
# or in Nix shell:
nix develop -c cargo run -p xtask -- selftest
```

**Behavior:**
1. Runs `xtask check` (format, clippy, tests)
2. Runs `xtask bdd` (acceptance tests + JUnit XML)
3. Runs `xtask ac-status` (AC status mapping)
4. Runs `xtask bundle implement_ac` (LLM bundler)
5. Runs `xtask policy-test` (Rego policy tests if conftest available)
6. Reports comprehensive validation results with colored output

**Exit codes:**
- `0`: All self-tests passed
- Non-zero: One or more test suites failed

**Output artifacts:**
- `target/junit/acceptance.xml`: JUnit test results
- `docs/feature_status.md`: AC status mapping
- `.llm/bundle/implement_ac.md`: LLM context bundle

**Use case:**
- **CI/CD pipelines**: Single command for comprehensive validation
- **Pre-release checks**: Verify template health before tagging
- **Development**: Full validation after major changes

**Notes:**
- This is the canonical CI command (used in `ci-template-selftest.yml`)
- AC status and policy checks are informational (don't fail suite if unavailable)
- More comprehensive than `xtask quickstart` (which is for first-run validation)

---

### `xtask quickstart`

Quick validation of all template functionality.

**Usage:**
```bash
cargo run -p xtask -- quickstart
```

**Behavior:**
1. Checks environment (cargo, rustc versions)
2. Runs `xtask check` (format, clippy, tests)
3. Runs `xtask bdd` (acceptance tests + JUnit XML)
4. Runs `xtask bundle implement_ac` (LLM bundler)
5. Reports validation results with colored output

**Exit codes:**
- `0`: All validation steps passed
- Non-zero: One or more validation steps failed

**Use case:**
- First-time template validation after cloning
- Verifying template setup before starting work
- CI self-test for template repository

**Notes:**
- This is the recommended first command to run after cloning
- Provides clear visual feedback with ✓/✗/⚠ indicators
- Fails fast if environment is not properly configured

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

## LLM Context Bundler (Rust-native)

**Purpose:** Generate focused context bundles for LLM consumption.

**Implementation:** `crates/xtask/src/commands/bundle.rs`

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
1. Parses task configuration from `.llm/contextpack.yaml` using `serde_yaml`
2. Resolves `include` patterns via `git ls-files` for each pattern
3. Applies `.llm/.llmignore` exclusion patterns
4. Deduplicates files matched by multiple patterns
5. Concatenates files with `# FILE: path` headers in markdown format
6. Enforces `max_bytes` limit (stops adding files when limit reached)
7. Writes to `.llm/bundle/<task>.md`

**Output format:**
```markdown
# LLM Context Bundle

**Git SHA:** <commit-sha>

**Description:** <task description>

**Max bytes:** <limit>

---

# FILE: path/to/file

```
<file contents>
```

# FILE: another/file

```
<file contents>
```
```

**Exit codes:**
- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

**Usage:**
```bash
cargo run -p xtask -- bundle <task>
```

---

### `.llmignore` File Semantics

**Location:** `.llm/.llmignore`

**Purpose:** Exclude files from LLM context bundles using pattern matching.

**Current Implementation:** Minimal pattern matching (see design doc for planned gitignore semantics adoption)

**Supported Patterns:**

1. **Exact component match:** `foo` matches any path component named exactly "foo"
   - Matches: `foo`, `bar/foo`, `bar/foo/baz.txt`
   - Does NOT match: `foobar`, `foo.txt`

2. **Directory pattern:** `foo/` matches directory and its contents
   - Matches: `foo/bar.txt`, `foo/baz/qux.rs`
   - Does NOT match: `bar/foo/baz.txt` (only at root currently)

3. **Comments:** Lines starting with `#` are ignored

4. **Whitespace:** Leading/trailing whitespace is trimmed

**Example `.llm/.llmignore`:**
```
# Ignore build artifacts
target/
dist/

# Ignore specific files
Cargo.lock
.DS_Store

# Ignore test directories
tests/
```

**Limitations (Current Implementation):**
- No glob patterns (`*.log`, `test_*.rs`)
- No wildcards (`?`, `*`, `**`)
- No path anchoring (`/foo` vs `foo`)
- No negation patterns (`!important.log`)
- No character classes (`[0-9]`)

**Planned Enhancement:**
Full gitignore semantics via the `ignore` crate. See `docs/design/llmignore-semantics.md` for analysis and implementation plan.

**Processing Order:**
1. Load patterns from `.llm/.llmignore`
2. Filter comments and empty lines
3. Apply patterns to files matched by `include` globs
4. Excluded files are not added to bundle

**Notes:**
- Patterns are applied after `git ls-files` resolves include globs
- Files must be git-tracked to be considered for bundling
- Exclusion happens before size limit enforcement

---

## Test Scripts (Legacy)

### `scripts/test-policies.sh`

**DEPRECATED:** Use `cargo run -p xtask -- policy-test` instead.

Legacy bash wrapper for testing Rego policies. Kept for backward compatibility.

**Usage:**
```bash
bash scripts/test-policies.sh
```

**Exit codes:**
- `0`: All policy tests passed
- Non-zero: One or more policy tests failed

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
