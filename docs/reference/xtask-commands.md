# Reference: xtask Commands
<!-- doclint:disable orphan-version -->

Complete reference for all `xtask` CLI commands.

**JSON Output Support:** The following commands support `--json` flag for agent/portal integration:
- `ac-status --json` - AC coverage report as structured JSON
- `friction-list --json` - Friction entries with statistics
- `questions-list --json` - Questions with statistics
- `fork-list --json` - Fork registry with kernel version breakdown
- `version --json` - Kernel and template version information

**Quick Index:**
- [dev-up](#xtask-dev-up) - One-command environment setup
- [status](#xtask-status) - Governance status dashboard
- [check](#xtask-check) - Format, lint, test
- [bdd](#xtask-bdd) - Run BDD acceptance tests
- [ac-status](#xtask-ac-status) - Generate AC status report
- [ac-coverage](#xtask-ac-coverage) - Show AC coverage and unknown ACs
- [ac-suggest-scenarios](#xtask-ac-suggest-scenarios) - Generate BDD scenario stub
- [policy-test](#xtask-policy-test) - Test Rego policies
- [bundle](#xtask-bundle) - Generate LLM context
- [release-bundle](#xtask-release-bundle) - Generate release evidence bundle
- [skills-fmt](#xtask-skills-fmt) - Normalize SKILL.md files
- [skills-lint](#xtask-skills-lint) - Validate Skills definitions
- [quickstart](#xtask-quickstart) - First-run validation
- [selftest](#xtask-selftest) - Comprehensive 11-step validation suite
- [idp-check](#xtask-idp-check) - Validate IDP/portal integration surface
- [contracts-check](#xtask-contracts-check) - Validate governed facts match docs
- [contracts-fmt](#xtask-contracts-fmt) - Sync governed facts to docs
- [ui-contract-check](#xtask-ui-contract-check) - Validate UI contract and DOM anchors

---

## xtask dev-up

One-command environment setup and health check.

### Usage

```bash
cargo run -p xtask -- dev-up

# Or in Nix shell
nix develop -c cargo run -p xtask -- dev-up
```

### What It Does

Performs a comprehensive environment setup and validation:

1. **Dependency check:** Verifies required tools (cargo, rustc, conftest, etc.)
2. **Platform health:** Checks if HTTP server is running (`http://localhost:8080/platform/status`)
3. **Governance validation:** Parses specs and verifies structure
4. **Core checks:** Runs `xtask check` (fmt, clippy, tests)
5. **BDD tests:** Runs acceptance scenarios
6. **Guidance:** Shows next steps and helpful URLs

### Exit Codes

- `0`: Environment ready
- Non-zero: Setup issues detected

### When to Use

- **First time** - After cloning the repository
- **After environment changes** - Tool updates, config changes
- **Daily standup** - Quick "am I ready to code?" check
- **For agents** - Automated environment verification

### Example Output

```
======================================
  Dev Environment Setup
======================================

[1/6] Checking dependencies...
  ✓ cargo 1.91.0
  ✓ rustc 1.91.0
  ✓ conftest 0.52.0

[2/6] Checking platform health...
  ✓ Platform running at http://localhost:8080
  ✓ /platform/status responding

[3/6] Validating governance...
  ✓ spec_ledger.yaml parsed
  ✓ tasks.yaml parsed

[4/6] Running core checks...
  ✓ Format check passed
  ✓ Clippy passed
  ✓ Tests passed

[5/6] Running BDD tests...
  ✓ All scenarios passed

[6/6] Checking commands...
  ✓ All xtask commands available

======================================
✓ Environment ready!

Next steps:
  • View governance: http://localhost:8080/ui
  • Check status: cargo xtask status
  • List tasks: cargo xtask tasks-list
  • Run selftest: cargo xtask selftest
======================================
```

### Common Issues

**Platform not running:**
```bash
# Start the platform first
cargo run -p app-http &

# Then run dev-up
cargo xtask dev-up
```

**Missing dependencies:**
```bash
# Enter Nix shell (recommended)
nix develop

# Or install tools manually
brew install conftest  # macOS
```

**Checks fail:**
- Run `cargo xtask check` separately to see detailed errors
- Fix issues and re-run `dev-up`

---

## xtask status

Show governance status dashboard (CLI summary).

### Usage

```bash
cargo run -p xtask -- status

# Or with alias
xt status
```

### What It Does

Displays a quick governance health snapshot:

1. Reads `specs/spec_ledger.yaml` to count Stories, Requirements, ACs
2. Reads `specs/tasks.yaml` to count tasks by status (Todo, InProgress, Review, Done)
3. Shows template version from ledger metadata
4. Provides helpful next-step commands

### Exit Codes

- `0`: Always succeeds (read-only operation)

### When to Use

- **Quick orientation** - "What's the state of this cell?"
- **Morning standup** - Check task counts without opening UI
- **Agent workflows** - Lightweight status check before planning work
- **CI reporting** - Include in build logs for visibility

### Example Output

```
======================================
Rust-as-Spec – 2.4.0
======================================

Governance:
  Stories:      3
  Requirements: 23
  ACs:          46

Tasks:
  Todo:        3
  InProgress:  2
  Review:      1
  Done:       15

Next steps:
  • View tasks:     cargo xtask tasks-list
  • Run selftest:   cargo xtask selftest
  • Start platform: cargo run -p app-http
  • View UI:        http://localhost:8080/ui
======================================
```

### Notes

- **Read-only:** Never modifies files or state
- **Fast:** Parses YAML files only, no network calls
- **No dependencies:** Works even if platform is offline
- **Complements `/platform/status`:** CLI equivalent of HTTP endpoint

### Difference from `/platform/status`

| Feature | `xtask status` | `/platform/status` |
|---------|----------------|-------------------|
| Type | CLI tool | HTTP API |
| Data source | YAML files | In-memory runtime state |
| Requires platform | No | Yes |
| Use case | Quick CLI check | Programmatic access |

---

## xtask check

Run all code quality checks: formatting, linting, and tests.

### Usage

```bash
cargo run -p xtask -- check

# Or in Nix shell
nix develop -c cargo run -p xtask -- check
```

### What It Does

Runs three checks in sequence:

1. **Format check:** `cargo fmt --all -- --check`
2. **Lint check:** `cargo clippy --all-targets --all-features -- -D warnings`
3. **Unit tests:** `cargo test --workspace --exclude acceptance`

Stops at first failure.

### Exit Codes

- `0`: All checks passed
- Non-zero: One or more checks failed

### When to Use

- **Before every commit** - Ensure code quality
- **In pre-commit hooks** - Automatic validation
- **In CI** - Required check for pull requests

### Example Output

```
Running format check...
Running clippy...
Running tests...
   Running unittests src/lib.rs (target/debug/deps/core-...)
   Running unittests src/lib.rs (target/debug/deps/model-...)
✓ All checks passed
```

### Common Issues

**Format check fails:**
```bash
# Fix by running
cargo fmt --all
```

**Clippy warnings:**
```bash
# See specific warnings
cargo clippy --all-targets --all-features
```

**Tests fail:**
```bash
# Run tests with output
cargo test --workspace --exclude acceptance -- --nocapture
```

---

## xtask bdd

Run BDD acceptance tests with JUnit XML output.

### Usage

```bash
cargo run -p xtask -- bdd
```

### What It Does

1. Runs `cargo test -p acceptance --test acceptance`
2. Generates JUnit XML at `target/junit/acceptance.xml`
3. Prints scenario results to console

### Exit Codes

- `0`: All scenarios passed
- Non-zero: One or more scenarios failed

### Output Artifacts

- `target/junit/acceptance.xml` - JUnit test results for CI

### When to Use

- **After implementing AC** - Verify scenario passes
- **Before pull request** - Ensure acceptance criteria met
- **In CI** - Validate behavioral requirements

### Example Output

```
Running acceptance tests...
Feature: Platform Status
  Scenario: Platform status returns governance health
   ✔  Given the service is running
   ✔  When I GET /platform/status
   ✔  Then I receive 200 with governance data
[Summary]
1 feature
1 scenario (1 passed)
3 steps (3 passed)
✓ Acceptance tests passed
JUnit output: target/junit/acceptance.xml
```

### Common Issues

**Scenario fails:**
- Check step definitions in `crates/acceptance/src/steps/`
- Verify feature file syntax in `specs/features/`
- Run with verbose output: `cargo test -p acceptance -- --nocapture`

**JUnit XML not generated:**
- Check `target/junit/` directory exists
- Verify path in `crates/acceptance/tests/acceptance.rs`

---

## xtask ac-status

Generate AC status report from acceptance test results.

### Usage

```bash
cargo run -p xtask -- ac-status

# Output as JSON for agent/portal integration
cargo run -p xtask -- ac-status --json

# Show details for a single AC
cargo run -p xtask -- ac-status --ac AC-KERN-001

# Single AC as JSON (useful for debugging)
cargo run -p xtask -- ac-status --ac AC-KERN-001 --json

# Or in Nix shell
nix develop -c cargo run -p xtask -- ac-status
```

### Parameters

| Flag | Description |
|------|-------------|
| `--json` | Output structured JSON instead of generating markdown |
| `--ac <ID>` | Show details for a single AC (e.g., `AC-KERN-001`) |
| `--summary` | Print concise summary to stdout |

### What It Does

1. Reads `specs/spec_ledger.yaml` to extract all AC definitions
2. **Primary:** Parses AC coverage JSONL (`target/ac/coverage.jsonl`) written by the BDD harness
   - Streams results, resilient to `std::process::exit()` (cucumber-rs issue)
   - See `docs/design/ac-coverage-format.md` for format specification
3. **Fallback 1:** JSON report (`target/ac_report.json`) if coverage.jsonl unavailable
4. **Fallback 2 (legacy):** JUnit XML + feature file parsing if JSON unavailable
   - Parses `specs/features/**/*.feature` for `@AC-####` tags
   - Parses `target/junit/acceptance.xml` for test results
   - May be unreliable due to cucumber-rs exit() behavior
5. Maps scenarios → ACs based on `@AC-####` tags
6. Computes status for each AC (pass/fail/unknown)
7. Generates `docs/feature_status.md` with status table

**Note:** The coverage.jsonl path is the recommended approach. JUnit fallback is for backward compatibility and may be removed in a future major version.

### JSON Output (Schema v2.0)

When `--json` is specified, outputs structured JSON instead of generating the markdown file.

**Schema version 2.0** uses `must_have_ac` metadata for AC classification instead of prefix-based heuristics:

```json
{
  "schema_version": "2.0",
  "timestamp": "2025-12-05T12:00:00Z",
  "must_have_acs": {
    "total": 48,
    "passing": 46,
    "failing": 1,
    "unknown": 1
  },
  "optional_acs": {
    "total": 17,
    "passing": 15,
    "failing": 0,
    "unknown": 2
  },
  "coverage_percent": 93.8,
  "acs": [
    {
      "id": "AC-TPL-001",
      "story_id": "US-TPL-001",
      "req_id": "REQ-TPL-HEALTH",
      "text": "Doctor command validates environment",
      "status": "pass",
      "source": "coverage",
      "must_have_ac": true,
      "scenarios": ["Doctor detects missing tools"],
      "tests": [...],
      "tests_total": 1,
      "tests_executed": 1
    }
  ]
}
```

#### JSON Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | string | Schema version (currently `"2.0"`). Bump on breaking changes. |
| `timestamp` | string | ISO 8601 timestamp of report generation |
| `must_have_acs` | object | Stats for ACs with `must_have_ac=true` (strictly enforced in selftest) |
| `optional_acs` | object | Stats for ACs with `must_have_ac=false` (informational) |
| `coverage_percent` | number | Overall coverage: (passing ACs / total ACs) × 100 |
| `acs` | array | Array of individual AC status objects |

**Per-AC Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | AC identifier (e.g., `AC-KERN-001`) |
| `story_id` | string | Parent user story ID |
| `req_id` | string | Parent requirement ID |
| `text` | string | Human-readable AC description |
| `status` | string | `"pass"`, `"fail"`, or `"unknown"` |
| `source` | string | Primary result source (see below) |
| `must_have_ac` | boolean | Whether AC participates in strict coverage gate |
| `scenarios` | array | BDD scenario names mapped to this AC |
| `tests` | array | Test mappings from ledger |
| `tests_total` | number | Total mapped tests declared in ledger |
| `tests_executed` | number | Tests that actually ran |

**Source Values:**

| Source | Description |
|--------|-------------|
| `"coverage"` | Result from `coverage.jsonl` (streaming BDD, preferred) |
| `"junit"` | Result from JUnit XML fallback |
| `"json"` | Result from Cucumber JSON fallback |
| `"inferred"` | No test results; status is ledger-only (`Unknown`) |

**`must_have_ac` Semantics:**

The `must_have_ac` flag uses AND semantics between requirement and AC:
- If `REQ.must_have_ac=true` AND `AC.must_have_ac=true` → effective `must_have_ac=true`
- If either is `false` → effective `must_have_ac=false`
- Both default to `true` if not specified in `spec_ledger.yaml`

When `XTASK_STRICT_AC_COVERAGE=1` is set, selftest fails if any `must_have_ac=true` AC has `status=unknown`.

### Exit Codes

- `0`: All ACs passed or unknown
- Non-zero: One or more ACs failed

### Output Artifacts

- `docs/feature_status.md` - AC status table with pass/fail/unknown indicators (not generated when --json is used)

### When to Use

- **After running BDD tests** - Check which ACs are covered
- **In CI** - Verify AC coverage
- **During development** - Understand test-to-AC mapping
- **Before releases** - Ensure all ACs have passing tests

### Example Output

```
Parsing ledger: specs/spec_ledger.yaml
  Found 3 ACs
Parsing JSON report: target/ac_report.json
  Found 3 scenarios
  Found results for 3 ACs
Generating status: docs/feature_status.md
✓ Generated docs/feature_status.md

✓ All ACs passed
```

**Legacy fallback output** (if JSON not available):
```
Parsing ledger: specs/spec_ledger.yaml
  Found 3 ACs
JSON report not found: target/ac_report.json
Falling back to JUnit + feature parsing (legacy)
  Found 3 scenarios
  Found results for 3 ACs
...
```

### Status Logic

- **Pass (✅)**: All mapped testcases passed
- **Fail (❌)**: Any mapped testcase failed
- **Unknown (❓)**: AC has no mapped scenarios or testcases

### Common Issues

**No JUnit XML found:**
- Run `cargo run -p xtask -- bdd` first to generate test results
- Check that `target/junit/acceptance.xml` exists

**ACs show as unknown:**
- Verify feature files have `@AC-####` tags
- Check that tag IDs match ledger AC IDs
- Ensure scenarios are actually running in BDD tests

**Unmapped scenarios:**
- Scenario has `@AC-####` tag that doesn't exist in ledger
- Check ledger for typos in AC IDs
- Add missing ACs to `specs/spec_ledger.yaml`

### Notes

- Normalizes testcase names by removing ` (row N)` and ` (example N)` suffixes
- Reports unmapped ACs (no scenarios) and unmapped scenarios (invalid AC refs)
- Used by `xtask selftest` and CI workflows

---

## xtask ac-coverage

Show AC coverage report grouped by requirement.

### Usage

```bash
cargo run -p xtask -- ac-coverage

# Show only ACs with Unknown status (coverage backlog)
cargo run -p xtask -- ac-coverage --todo

# Show only kernel (must_have_ac=true) ACs with Unknown status
cargo run -p xtask -- ac-coverage --todo --must-have

# Or with alias
xt ac-coverage
```

### Parameters

| Flag | Description |
|------|-------------|
| `--todo` | Show only ACs with Unknown status (coverage backlog checklist) |
| `--must-have` | When used with `--todo`, filter to only kernel ACs (`must_have_ac=true`) |

### What It Does

Displays AC coverage summary and identifies which ACs need BDD scenarios:

1. Reads `specs/spec_ledger.yaml` for all AC definitions
2. Parses feature files in `specs/features/` to find `@AC-####` tags
3. Reads test results from JSON or JUnit (same as `ac-status`)
4. Groups unknown ACs by requirement
5. Displays pass/fail/unknown counts and actionable next steps

The `--must-have` flag filters to only show "kernel" ACs where `must_have_ac=true` in the spec ledger. These are the ACs that selftest enforces coverage for.

### Exit Codes

- `0`: Always succeeds (read-only operation)

### When to Use

- **During AC development** - Identify which ACs still need scenarios
- **Sprint planning** - See coverage gaps at a glance (`--todo`)
- **Before releases** - Ensure all kernel ACs have coverage (`--todo --must-have`)
- **Onboarding new features** - Understand what's missing

### Example Output

```
📊 Computing AC coverage...

📋 AC Coverage Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ✓ 18 passing
  ✗ 0 failing
  ? 22 unknown (no BDD scenarios)

📍 Unknown ACs (Need BDD scenarios)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  REQ-PLT-ONBOARDING: AC-PLT-001, AC-PLT-002, AC-PLT-003, AC-PLT-018
  REQ-PLT-DESIGN-SCAFFOLDING: AC-PLT-004, AC-PLT-005
  REQ-PLT-RELEASE-SAFETY: AC-PLT-011, AC-PLT-012, AC-PLT-013

🎯 Suggested Next Steps
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  1. Generate scenario stub for AC-PLT-001:
     $ cargo xtask ac-suggest-scenarios AC-PLT-001

  2. Edit specs/features/your_feature.feature:
     $ vim specs/features/your_feature.feature

  3. Run BDD tests:
     $ cargo xtask bdd

  4. Check coverage again:
     $ cargo xtask ac-coverage
```

### Difference from `ac-status`

| Feature | `ac-coverage` | `ac-status` |
|---------|---------------|------------|
| **Purpose** | Quick coverage summary | Detailed AC status table |
| **Output** | Terminal summary | Markdown file |
| **Use case** | Sprint planning | CI/comprehensive validation |
| **Unknown ACs** | Grouped by REQ | Listed with text |

### Notes

- Groups unknown ACs by requirement ID for easy navigation
- Provides direct command suggestions for next steps
- No side effects (read-only operation)
- Fast execution (parses YAML + feature files)

---

## xtask ac-suggest-scenarios

Generate BDD scenario stub for an acceptance criterion.

### Usage

```bash
cargo run -p xtask -- ac-suggest-scenarios <AC-ID>

# Examples
cargo run -p xtask -- ac-suggest-scenarios AC-PLT-001
cargo run -p xtask -- ac-suggest-scenarios AC-TPL-002

# With alias
xt ac-suggest-scenarios AC-PLT-001
```

### What It Does

1. Reads `specs/spec_ledger.yaml` to find the AC text
2. Analyzes the AC text to suggest scenario structure
3. Generates a BDD scenario stub with appropriate Given/When/Then
4. Provides guidance for customization

### Parameters

- `<AC-ID>` - Acceptance Criterion ID (e.g., `AC-PLT-001`)

### Exit Codes

- `0`: Scenario stub generated successfully
- Non-zero: AC not found or invalid ID format

### When to Use

- **After creating a new AC** - Generate scenario template quickly
- **Before writing BDD tests** - Get a starting point
- **Batch scenario creation** - Streamline multiple ACs

### Example Output

```
🔧 Generating BDD scenario stub...

✓ Found AC: AC-PLT-001

Suggested BDD Scenario (add to specs/features/*.feature):

@AC-PLT-001
Scenario: xtask doctor validates Rust, Nix, conftest
  When I run the command
  Then [assertion about outcome]


Next steps:
  1. Copy the scenario above
  2. Edit specs/features/your_feature.feature
  3. Paste and customize the scenario:
     - Update When/Then steps to match the AC
     - Add specific test data or assertions
  4. Run: cargo xtask bdd
  5. Run: cargo xtask ac-status
```

### Scenario Template Logic

The command suggests steps based on AC text patterns:

| AC Text Contains | Suggested Step |
|------------------|----------------|
| "GET /health" | `When I make a GET request` |
| "POST /api" | `When I make a request` |
| "returns" | `Then the response should be valid` |
| "success" | `Then the operation should succeed` |
| "validates" or "checks" | Both Given and When suggested |
| "run" or "execute" | `When I run the command` |

### Best Practices

1. **Copy the stub** - The output is ready to paste into feature files
2. **Customize assertions** - Replace `[...]` placeholders with specific checks
3. **Use step definitions** - Check existing steps in `crates/acceptance/src/steps/`
4. **Keep it focused** - Each scenario should test one behavior
5. **Run immediately** - Test with `cargo xtask bdd` to validate syntax

### Example Workflow

```bash
# 1. Create an AC
cargo xtask ac-new AC-MY-001 "GET /api returns data" \
  --story US-MY-001 \
  --requirement REQ-MY-FEATURE

# 2. Generate scenario stub
cargo xtask ac-suggest-scenarios AC-MY-001

# 3. Edit the feature file and paste the stub
vim specs/features/my_feature.feature

# 4. Customize the scenario with real steps
# Edit: Given, When, Then to match the AC

# 5. Run BDD tests
cargo xtask bdd

# 6. Check coverage
cargo xtask ac-coverage
```

### Common Issues

**AC ID not found:**
- Verify AC exists in `specs/spec_ledger.yaml`
- Check ID format: must start with `AC-`
- Use exact case as in ledger

**Wrong scenario type suggested:**
- Template is a starting point only
- Always customize to match your specific AC
- Add multiple steps if needed

**Steps don't match AC:**
- Edit the Given/When/Then after pasting
- Add more specific assertions
- Reference actual API endpoints or commands

### Notes

- **Template-based** - Suggestion uses pattern matching, always review
- **Non-destructive** - Only outputs to terminal, no file changes
- **Quick iteration** - Regenerate with different AC ID easily
- **Pairing tool** - Works well with `ac-coverage` for batch flows

---

## xtask policy-test

Test Rego policies with conftest.

### Usage

```bash
cargo run -p xtask -- policy-test

# Or in Nix shell
nix develop -c cargo run -p xtask -- policy-test
```

### What It Does

Runs conftest policy tests for all policy areas:

1. **Ledger Policy** (`policy/ledger.rego`) - Ensures every AC has tests
2. **Features Policy** (`policy/features.rego`) - Validates feature-AC references
3. **Flags Policy** (`policy/flags.rego`) - Validates flag ownership and rollouts
4. **Privacy Policy** (`policy/privacy.rego`) - Ensures PII fields have owners and retention

Each policy is tested against fixtures in `policy/testdata/`:
- `{area}_valid.json` - Should pass
- `{area}_invalid.json` - Should fail
- `{area}_missing_tests.json` - Should fail (for ledger)
- `{area}_unknown_ac.json` - Should fail (for features)

### Exit Codes

- `0`: All policy tests passed
- Non-zero: One or more policy tests failed or conftest not available

### When to Use

- **During development** - Validate policy changes
- **Before commits** - Ensure policies still pass
- **In CI** - Governance validation
- **After adding ACs/flags/PII** - Verify metadata is complete

### Example Output

```
Testing Rego policies...

Ledger Policy (policy/ledger.rego):
  ✓ ledger_valid.json (correctly passed)
  ✓ ledger_missing_tests.json (correctly failed)

Features Policy (policy/features.rego):
  ✓ features_valid.json (correctly passed)
  ✓ features_unknown_ac.json (correctly failed)

Flags Policy (policy/flags.rego):
  ✓ flags_valid.json (correctly passed)
  ✓ flags_invalid.json (correctly failed)

Privacy Policy (policy/privacy.rego):
  ✓ privacy_valid.json (correctly passed)
  ✓ privacy_invalid.json (correctly failed)

✓ All 8 policy tests passed!
```

### Prerequisites

Requires `conftest` to be available on PATH:

**Install options:**
- **Nix:** `nix develop` (recommended - automatically available)
- **macOS:** `brew install conftest`
- **Linux:** See https://www.conftest.dev/install/
- **Container:** `docker run --rm openpolicyagent/conftest`

### Common Issues

**"conftest not found on PATH":**
- Enter Nix shell: `nix develop`
- Or install conftest manually for your platform

**Policy test fails unexpectedly:**
- Check fixture files in `policy/testdata/`
- Verify policy file syntax in `policy/*.rego`
- Run manually: `conftest test -p policy/ledger.rego policy/testdata/ledger_valid.json`

**No test fixtures found:**
- Check that `policy/testdata/{area}_valid.json` exists
- Policy will skip if no fixtures found

### Notes

- Part of `xtask selftest` but gracefully degrades if conftest unavailable
- Each policy area is tested independently
- Fixtures use realistic data structures from actual specs

---

## xtask bundle

Generate LLM context bundle for a specific task.

### Usage

```bash
cargo run -p xtask -- bundle <task>

# Examples
cargo run -p xtask -- bundle implement_ac
cargo run -p xtask -- bundle implement_feature
cargo run -p xtask -- bundle debug_tests
```

### What It Does

1. Reads `.llm/contextpack.yaml` for task configuration
2. Resolves `include` glob patterns via `git ls-files`
3. Respects `.llm/.llmignore` exclusions (using gitignore syntax)
4. Enforces `max_bytes` limit
5. Generates markdown bundle at `.llm/bundle/<task>.md`

### Parameters

- `<task>` - Task name defined in `.llm/contextpack.yaml`

### Exit Codes

- `0`: Bundle generated successfully
- Non-zero: Task not found or bundling failed

### Output Artifacts

- `.llm/bundle/<task>.md` - Generated context bundle

### When to Use

- **Before LLM coding session** - Get focused context
- **When implementing AC** - Provide specs + tests + code
- **When debugging** - Include relevant test failures

### Example Output

```
Generating LLM context bundle for task: implement_ac
Building context bundle: implement_ac
  Max size: 250000 bytes
  Description: Context for implementing an AC: ledger, specs, features, and core code
  Files included: 6
  Bundle size: 2708 bytes

Bundle written to: /path/to/.llm/bundle/implement_ac.md
✓ Bundle generated: .llm/bundle/implement_ac.md
```

### Common Issues

**Task not found:**
- Check `.llm/contextpack.yaml` has task defined
- Verify task name spelling

**Bundle too large:**
- Reduce `max_bytes` in contextpack.yaml
- Make `include` patterns more specific

**Missing files:**
- Files must be tracked by git (`git ls-files`)
- Check `.llm/.llmignore` isn't excluding needed files (uses gitignore syntax)

### Configuration

Edit `.llm/contextpack.yaml`:

```yaml
tasks:
  my_task:
    max_bytes: 150000
    include:
      - specs/spec_ledger.yaml
      - crates/core/src/**/*.rs
    description: "Custom task description"
```

---

## xtask release-bundle

Generate a structured release evidence bundle for a given version.

### Usage

```bash
cargo run -p xtask -- release-bundle <version>

# Example
cargo run -p xtask -- release-bundle 3.3.6
```

### What It Does

Creates a comprehensive evidence file at `release_evidence/v<version>.md` containing:

1. **Tasks Completed** - All tasks with status "done" from `specs/tasks.yaml`
2. **Acceptance Criteria & Requirements** - Linked REQs/ACs from `specs/spec_ledger.yaml`
3. **Architecture Decisions** - List of ADRs from `docs/adr/`
4. **Git Changelog** - Commit log since last git tag
5. **Governance Status** - Selftest results and policy status
6. **Resolved Friction** - Entries marked as resolved in `FRICTION_LOG.md`

### Parameters

- `<version>` - Version number (e.g., `3.3.6`) - format should be `X.Y.Z`

### Exit Codes

- `0`: Evidence bundle generated successfully
- Non-zero: Invalid version format or generation failed

### Output Artifacts

- `release_evidence/v<version>.md` - Complete evidence bundle for the release

### When to Use

- **Before cutting a release** - Generate evidence to review what's included
- **For changelog generation** - Feed evidence file to LLM for Keep a Changelog format
- **For release notes** - Use as source material for GitHub releases
- **For compliance** - Provide auditable evidence of what changed
- **Post-release** - Commit evidence file for historical tracking

### Example Output

```
📦 Generating release evidence bundle for 3.3.6...

📋 Collecting evidence...

✓ Evidence bundle written to: /path/to/release_evidence/v3.3.6.md

Next steps:
  1. Review evidence: cat release_evidence/v3.3.6.md
  2. Feed to LLM for changelog generation
  3. Update CHANGELOG.md with generated content
```

### Evidence File Structure

The generated markdown file includes:

```markdown
# Release Evidence: v3.3.6

**Generated:** 2025-01-21 10:30:00

---

## Tasks Completed

**Total completed:** 3 tasks

### TASK-TPL-REL-BUNDLE-3-3-6

**Title:** Implement release-bundle command
**Requirement:** REQ-TPL-REL-BUNDLE
**ACs:** AC-TPL-REL-EVIDENCE, AC-TPL-REL-CHANGELOG
...

## Acceptance Criteria & Requirements

### REQ-TPL-REL-BUNDLE - Release evidence bundle generation

**Story:** US-TPL-PLT-001 - Platform: Developer Experience & Governance
**Tags:** platform, release, devex
...

## Architecture Decisions

**Total ADRs:** 7

- 0001-adopt-hexagonal-architecture.md
- 0002-nix-first-development.md
...

## Git Changelog

**Since tag:** v2.4.0

- abc1234 feat: add release-bundle command (Agent)
- def5678 docs: update roadmap for v3.3.x (User)
...

## Governance Status

### Selftest Status

**Status:** ✅ PASSED

```
[7/7] Validating graph invariants...
  ✓ Graph invariants validated

======================================
✓ All 7 steps passed!
======================================
```

### Policy Status

```json
{"status": "pass", "tests": 8, "failures": 0}
```

## Resolved Friction

**Total resolved entries:** 2

### FE-001: Port confusion between 8080 and 3000
...
```

### Integration with Release Workflow

Typical release workflow:

```bash
# 1. Prepare release (update versions)
cargo xtask release-prepare 3.3.6

# 2. Generate evidence bundle
cargo xtask release-bundle 3.3.6

# 3. Review evidence and generate changelog
cat release_evidence/v3.3.6.md
# Feed to LLM: "Generate Keep a Changelog entry from this evidence"

# 4. Update CHANGELOG.md with LLM-generated content

# 5. Verify release readiness
cargo xtask release-verify

# 6. Commit and tag
git add .
git commit -m "Release v3.3.6"
git tag -a v3.3.6 -m "Release 3.3.6"
git push && git push --tags
```

### LLM Changelog Generation

The evidence bundle is designed to be fed to an LLM with a prompt like:

```
Given this release evidence bundle, generate a CHANGELOG.md entry
in Keep a Changelog format (Added/Changed/Fixed/Removed sections).
Focus on user-visible changes and group related items logically.

[Paste evidence bundle content]
```

The LLM can then:
- Group tasks by category (Added/Changed/Fixed/Removed)
- Convert technical AC language to user-friendly descriptions
- Identify breaking changes from git log
- Generate concise bullet points
- Add links to issues/PRs if present in git log

### Common Issues

**No completed tasks found:**
- Ensure tasks in `specs/tasks.yaml` have `status: done` (lowercase)
- Check that tasks were completed for this release
- Manually mark tasks as done if needed

**Git log empty:**
- Ensure you've made commits since the last tag
- Check that git repository has at least one tag
- First release: git log will show all commits

**Selftest fails during evidence generation:**
- Evidence bundle will still be created, but governance status will show failure
- Fix selftest issues before releasing
- Run `cargo xtask selftest` separately to diagnose

**Policy status not found:**
- Run `cargo xtask policy-test` to generate `target/policy_status.json`
- Policy status section will note if file is missing

### Notes

- **Idempotent:** Running multiple times overwrites the evidence file
- **Version-agnostic tasks:** Currently filters by status, not version field
- **Git-dependent:** Requires git repository with at least one tag for changelog
- **Selftest integration:** Runs selftest in low-resource mode for status
- **Historical record:** Commit evidence files for audit trail

---

## xtask skills-fmt

Normalize Agent Skills definitions in `.claude/skills/*/SKILL.md`.

### Usage

```bash
cargo run -p xtask -- skills-fmt
```

### What It Does

Applies repository-wide formatting rules to Skills definitions:

1. Locates all `SKILL.md` files under `.claude/skills/`
2. Parses YAML frontmatter with `serde_yaml`
3. Normalizes field order, required fields, and spacing
4. Ensures consistent formatting across all Skills

### Exit Codes

- `0`: All Skills formatted successfully
- Non-zero: Formatting failed or invalid Skills found

### When to Use

- **Before committing Skills changes** - Ensure consistency
- **After creating/editing Skills** - Normalize formatting
- **In pre-commit hooks** - Automatic formatting

### Example Output

```
🎨 Formatting Agent Skills...

Formatted:
  ✓ .claude/skills/governed-feature-dev/SKILL.md
  ✓ .claude/skills/governed-maintenance/SKILL.md
  ✓ .claude/skills/governed-release/SKILL.md

✓ 3 Skills formatted
```

### Common Issues

**Permission errors:**
- Ensure Skills files are writable
- Check file permissions in `.claude/skills/`

**Invalid YAML:**
- Fix YAML syntax in frontmatter
- Verify frontmatter is enclosed in `---` delimiters

### Notes

- **Idempotent:** Safe to run multiple times
- **Preserves content:** Only formats, doesn't change meaning
- **Part of governance:** Skills are governed artifacts

---

## xtask skills-lint

Lint Agent Skills definitions for structural and governance correctness.

### Usage

```bash
cargo run -p xtask -- skills-lint
```

### What It Checks

Validates Skills definitions against repository standards:

1. **Required frontmatter fields** - `name`, `description` exist
2. **Name conventions** - Kebab-case, max length
3. **Description quality** - Includes "what" and "when to use"
4. **References** - Links to flows (`ac_first`, `onboarding`) and/or xtask commands
5. **Location** - Skills live under `.claude/skills/` only

### Exit Codes

- `0`: All Skills valid
- Non-zero: One or more Skills invalid; errors printed

### When to Use

- **Before committing Skills changes** - Validate structure
- **In CI** - Prevent invalid Skills from landing
- **When adding/refactoring Skills** - Ensure compliance
- **In pre-commit hooks** - Automatic validation

### Example Output

```
🔍 Linting Agent Skills...

Checking:
  ✓ .claude/skills/governed-feature-dev/SKILL.md
  ✓ .claude/skills/governed-maintenance/SKILL.md
  ✗ .claude/skills/governed-release/SKILL.md
    → Missing description in frontmatter
    → Description must include "when to use"

✗ 1/3 Skills failed validation
```

### Common Issues

**Missing frontmatter fields:**
- Add required fields: `name`, `description`
- Ensure frontmatter is valid YAML

**Invalid name format:**
- Use kebab-case: `governed-feature-dev`, not `GovernedFeatureDev`
- Keep names concise (max 50 characters)

**Description issues:**
- Include both what the Skill does and when to use it
- Reference flows from `specs/devex_flows.yaml`

**No workflow references:**
- Link to flows: `ac_first`, `onboarding`, etc.
- Or reference xtask commands: `cargo xtask check`

### Integration

Part of `cargo xtask docs-check`:

```bash
# Runs as part of documentation validation
cargo run -p xtask -- docs-check

# Output includes Skills check:
# Skills definitions... ✓ Valid
```

### Notes

- **Non-invasive:** Read-only, doesn't modify files
- **Use with skills-fmt:** Run `skills-fmt` to fix formatting first
- **Governance integration:** Skills are governed artifacts like docs

---

## xtask quickstart

Quick validation of template functionality (first-run check).

### Usage

```bash
cargo run -p xtask -- quickstart
```

### What It Does

1. Checks environment (cargo, rustc versions)
2. Runs `xtask check`
3. Runs `xtask bdd`
4. Runs `xtask bundle implement_ac`
5. Reports results with colored output

### Exit Codes

- `0`: All validation passed
- Non-zero: One or more validation steps failed

### When to Use

- **First time** - After cloning template
- **After setup changes** - Verify environment still works
- **Quick health check** - Lighter than `selftest`

### Example Output

```
======================================
  Rust Template Quick Start
======================================

[1/5] Checking environment...
  ✓ cargo 1.91.0
  ✓ rustc 1.91.0

[2/5] Running xtask check...
  ✓ Format check passed
  ✓ Clippy passed
  ✓ Tests passed

[3/5] Running BDD acceptance tests...
  ✓ BDD scenarios passed
  ✓ JUnit output created

[4/5] Testing LLM context bundler...
  ✓ Bundle command executed
  ✓ Bundle created (2708 bytes)

[5/5] Testing helper commands...
  ✓ Core commands validated

======================================
✓ Template validation passed!

Next steps:
  • See docs/how-to/new-service-from-template.md for adoption guide
  • See TEMPLATE_API.md for stable interface documentation
  • See docs/tutorials/first-ac-change.md for AC-first development
======================================
```

### Difference from `selftest`

| Feature | quickstart | selftest |
|---------|-----------|----------|
| Environment check | ✓ | ✓ |
| Core checks | ✓ | ✓ |
| BDD tests | ✓ | ✓ |
| AC status | ✗ | ✓ |
| Policy tests | ✗ | ✓ (if available) |
| **Use case** | First run | CI/comprehensive |

---

## xtask selftest

Complete template self-test suite (used in CI). **This is the governance gate** - if selftest passes, your changes are valid.

### Usage

```bash
cargo run -p xtask -- selftest

# Low-resource mode (for CI/constrained environments)
XTASK_LOW_RESOURCES=1 cargo run -p xtask -- selftest

# Verbose mode
cargo run -p xtask -- selftest -v

# Strict AC coverage mode (fails on Unknown kernel ACs)
XTASK_STRICT_AC_COVERAGE=1 cargo run -p xtask -- selftest
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `XTASK_LOW_RESOURCES=1` | Reduce parallelism and skip resource-intensive checks |
| `XTASK_SKIP_BDD=1` | Skip BDD tests (used internally to avoid recursion) |
| `XTASK_STRICT_AC_COVERAGE=1` | Fail selftest if kernel (`must_have_ac=true`) ACs have Unknown status (no BDD coverage) |

### What It Does

Comprehensive validation in **11 steps**:

1. **Core checks:** format, clippy, tests
2. **BDD tests:** acceptance scenarios + JUnit XML
3. **AC/ADR mapping:** Validates traceability (tests → ACs, REQs → ADRs)
4. **LLM bundler:** Validates context generation
5. **Policy tests:** Runs Rego policies (if conftest available)
6. **DevEx contract:** Verifies required commands exist
7. **Graph invariants:** Validates structural integrity (no orphaned REQs, missing ACs)

### Exit Codes

- `0`: All self-tests passed
- Non-zero: One or more test suites failed

### Output Artifacts

- `target/junit/acceptance.xml` - JUnit test results
- `docs/feature_status.md` - AC status mapping
- `.llm/bundle/implement_ac.md` - LLM context bundle

### When to Use

- **In CI/CD** - Comprehensive validation
- **Before releases** - Full health check
- **After major changes** - Verify nothing broke

### Example Output (Normal Mode)

```
======================================
  Template Self-Test Suite
======================================

[1/7] Running core checks (fmt, clippy, tests)...
  ✓ Core checks passed

[2/7] Running BDD acceptance tests...
  ✓ BDD scenarios passed

[3/7] Checking AC/ADR mapping...
  ✓ AC status mapping verified
  ✓ ADR references validated

[4/7] Testing LLM context bundler...
  ✓ Bundle generated (2708 bytes)

[5/7] Running policy tests...
  ✓ Ledger policy passed
  ✓ LLM policy passed

[6/7] Checking DevEx contract...
  ✓ DevEx contract satisfied

[7/7] Validating graph invariants...
  ✓ Graph invariants validated

======================================
✓ All 7 steps passed!
======================================
```

### Example Output (Low-Resource Mode)

When running with `XTASK_LOW_RESOURCES=1`:

```
[1/7] Core checks...
  ✓ passed
[2/7] BDD...
  ✓ passed (summary output suppressed)
[3/7] AC/ADR mapping...
  ✓ passed
[4/7] LLM bundler...
  ✓ passed
[5/7] Policy tests...
  ✓ passed
[6/7] DevEx contract...
  ✓ passed
[7/7] Graph invariants...
  ✓ passed

======================================
✓ All 7 steps passed!
======================================
```

**Low-resource mode benefits:**
- Reduced output (less logging from BDD runner)
- Same validation rigor
- Faster in CI environments
- Easier to parse for automation

### CI Integration

Used in `.github/workflows/ci-template-selftest.yml`:

```yaml
- name: Run template self-test suite
  run: nix develop -c cargo run -p xtask -- selftest
```

### Graceful Degradation

- AC status failures are warnings (informational)
- Policy tests skip if conftest unavailable
- Only core checks and BDD are hard requirements

---

## IDP / Portal Integration

### xtask idp-check

Validate IDP integration surface (OpenAPI lint + Backstage plugin checks).

#### Usage

```bash
cargo run -p xtask -- idp-check

# Verbose output
cargo run -p xtask -- idp-check -v
```

#### What It Does

Runs focused validation for IDP/portal integration:

1. **OpenAPI lint**: Runs Redocly linting on `specs/openapi/openapi.yaml`
2. **Backstage plugin checks**: Runs `pnpm run check` in `examples/backstage-plugin/`
3. **TypeScript config validation**: Validates tsconfig.json against governance rules
4. **PlatformClient tests**: Runs TypeScript tests for API contract compliance

#### Exit Codes

- `0`: All IDP checks passed
- Non-zero: One or more checks failed

#### When to Use

- **After OpenAPI changes**: Verify schema is still valid and TypeScript types compile
- **After platform endpoint changes**: Ensure Backstage plugin still type-checks
- **Before releases**: Validate the IDP surface is healthy
- **During Backstage integration work**: Quick feedback loop

#### Example Output

```
🔍 Validating IDP integration surface...

[1/3] Running OpenAPI lint...
  ✓ specs/openapi/openapi.yaml valid

[2/3] Running Backstage plugin checks...
  ✓ TypeScript compilation passed
  ✓ Type tests passed

[3/3] Validating TypeScript config...
  ✓ No deprecated moduleResolution
  ✓ No ignoreDeprecations flags

✓ IDP surface validation passed
```

#### Notes

- **Non-kernel**: This is an ergonomic helper for IDP integration, not a kernel requirement
- **Requires pnpm**: Backstage plugin checks need pnpm installed
- **Complements selftest**: Use `selftest` for full governance; use `idp-check` for focused IDP validation

---

## Contracts Governance

### xtask contracts-check

Validate that governed facts in documentation match their sources.

#### Usage

```bash
cargo run -p xtask -- contracts-check
```

#### What It Does

Validates governed facts against their sources:

1. **Computes facts** from code and specs:
   - Selftest step count (from `[N/M]` patterns in `selftest.rs`)
   - AC counts by classification (kernel/template/meta from `spec_ledger.yaml`)
   - Platform endpoints (from `openapi.yaml`)
   - Required checks (from `devex_flows.yaml`)

2. **Loads patterns** from `specs/contracts_manifest.yaml`

3. **Scans documentation** for patterns that contain governed numbers

4. **Reports drift** if any documented numbers don't match computed values

#### Exit Codes

- `0`: All documented facts match sources
- Non-zero: Drift detected (documented value differs from source)

#### When to Use

- **Before PR merge**: Ensure docs are synchronized
- **After changing selftest steps**: Verify step counts are updated
- **After AC changes**: Verify AC counts are updated
- **In CI**: Automated drift detection

#### Example Output

```
📋 Checking contract governance...

Computed facts from source:
  • Selftest steps: 11
  • AC counts: total=116, kernel=61, template=37, meta=18
  • Platform endpoints: 15
  • Required checks: 4

✓ All contract facts are synchronized
```

**When drift is detected:**

```
📋 Checking contract governance...

Computed facts from source:
  • Selftest steps: 11
  • AC counts: total=116, kernel=61, template=37, meta=18

Contract drift detected:

README.md:42
  Contract: selftest_step_count
  - An 10-step selftest gate
  + An 11-step selftest gate

Error: contracts-check found 1 edit(s) across 1 file(s). Run `cargo xtask contracts-fmt` to fix.
```

#### Common Issues

**Drift after adding selftest step:**
- You added a step to `selftest.rs` but didn't update docs
- Run `cargo xtask contracts-fmt` to auto-fix

**AC count mismatch:**
- You changed ACs in `spec_ledger.yaml` but `feature_status_notes.md` is stale
- Run `cargo xtask contracts-fmt` to auto-fix

**Pattern not matching:**
- The regex in `contracts_manifest.yaml` doesn't match the doc format
- Update the regex or adjust the documentation format

---

### xtask contracts-fmt

Synchronize governed facts from sources to documentation.

#### Usage

```bash
cargo run -p xtask -- contracts-fmt
```

#### What It Does

Updates documentation to match computed facts:

1. **Computes current values** from sources (same as `contracts-check`)
2. **Loads patterns** from `specs/contracts_manifest.yaml`
3. **Applies edits** to each file where documented values differ
4. **Reports changes** made

This is the "fix" command that pairs with `contracts-check`.

#### Exit Codes

- `0`: All files updated successfully (or no changes needed)
- Non-zero: Failed to apply edits

#### When to Use

- **After selftest step changes**: Update all step count references
- **After AC changes**: Update all AC count references
- **Before commits**: Auto-fix drift as part of workflow
- **In pre-commit hooks**: Automatic synchronization

#### Example Output

**When no changes needed:**

```
📋 Synchronizing contract facts...

Computed facts from source:
  • Selftest steps: 11
  • AC counts: total=116, kernel=61, template=37, meta=18
  • Platform endpoints: 15
  • Required checks: 4

✓ All contract facts are synchronized
```

**When changes are applied:**

```
📋 Synchronizing contract facts...

Computed facts from source:
  • Selftest steps: 11
  • AC counts: total=116, kernel=61, template=37, meta=18

  ✓ README.md
  ✓ docs/feature_status_notes.md

✓ Applied 2 contract updates
```

#### How Patterns Work

Patterns are defined in `specs/contracts_manifest.yaml`:

```yaml
contracts:
  selftest_step_count:
    source: "crates/xtask/src/commands/selftest.rs"
    description: "Number of steps in the selftest governance gate"
    patterns:
      - file: "README.md"
        regex: '(\d+)-step selftest gate'
        template: "{n}-step selftest gate"
```

- **regex**: Pattern to find the current value (capture group extracts number)
- **template**: Replacement text with `{n}` substituted

#### Configuration

Edit `specs/contracts_manifest.yaml` to add new patterns:

```yaml
contracts:
  my_fact:
    source: "path/to/source/file"
    description: "What this fact represents"
    patterns:
      - file: "docs/some-doc.md"
        regex: 'has (\d+) widgets'
        template: "has {n} widgets"
        required: false  # Don't fail if file doesn't exist
```

#### Common Issues

**Pattern doesn't match:**
- Regex is too strict or doc format changed
- Test regex against the actual line in the file
- Update `contracts_manifest.yaml` pattern

**Write permission denied:**
- Ensure documentation files are writable
- Check git isn't locking the files

#### Notes

- **Idempotent**: Safe to run multiple times
- **Atomic writes**: Uses temp file + rename to avoid partial writes
- **Part of docs-check**: `cargo xtask docs-check` runs `contracts-check` internally

---

### xtask ui-contract-check

Validate UI contract specification and DOM anchors.

#### Usage

```bash
cargo run -p xtask -- ui-contract-check
```

#### What It Does

Validates the UI contract system end-to-end:

1. **YAML Parse**: Loads `specs/ui_contract.yaml` and verifies structure
2. **Schema Validation**: Checks unique screen IDs, unique region IDs per screen, required fields
3. **Region Kind Refs**: Verifies all regions reference defined kinds in `region_kinds`
4. **DOM Validation**: Runs integration tests to verify HTML has matching `data-uiid` attributes

#### Exit Codes

- `0`: All UI contract checks passed
- Non-zero: One or more checks failed

#### When to Use

- **After changing `/ui` pages**: Verify DOM still matches contract
- **After editing `ui_contract.yaml`**: Validate contract structure
- **In CI**: Part of selftest step 9 (Governance graph & UI)
- **Before releases**: Ensure UI contract is synchronized

#### Example Output

```
🎨 Validating UI contract...

  [1/4] UI contract YAML ✓
  [2/4] Schema validation ✓
  [3/4] Region kind refs ✓
  [4/4] DOM validation tests ✓

UI contract validation PASSED
```

**When validation fails:**

```
🎨 Validating UI contract...

  [1/4] UI contract YAML ✓
  [2/4] Schema validation ✓
  [3/4] Region kind refs ✓
  [4/4] DOM validation tests ✗

UI contract validation FAILED

  ✗ DOM validation: Dashboard is missing data-uiid attributes for regions: ["dashboard.metrics"]

Fix the contract (specs/ui_contract.yaml) or HTML templates to match.
```

#### UI Contract Structure

The contract is defined in `specs/ui_contract.yaml`:

```yaml
schema_version: "1.0"
template_version: "v3.3.6"

screens:
  - id: platform_dashboard
    route: "/"
    aliases: ["/ui"]
    description: "Primary governance dashboard"
    regions:
      - id: "dashboard.health"
        kind: "panel"
        description: "Health metrics grid"
      - id: "dashboard.nav"
        kind: "navigation"
        description: "Main navigation bar"

region_kinds:
  panel: "Grouped content section"
  navigation: "Links to other screens"
```

#### DOM Anchors

HTML templates must include `data-uiid` attributes matching contract regions:

```rust
// In Maud templates
div data-uiid="dashboard.health" { /* content */ }
div data-uiid="dashboard.nav" { /* content */ }
```

The validation tests verify:
- Every region in the contract has a matching DOM element
- Routes are reachable (return 200 OK)
- Region IDs are unique per screen

#### API Endpoint

The contract is also exposed via HTTP at `/platform/ui/contract`:

```bash
curl http://localhost:8080/platform/ui/contract | jq
```

Returns the same structure as `specs/ui_contract.yaml` as JSON.

#### Integration with Selftest

This check runs as part of **step 9 (Governance graph & UI)** in selftest:

```
[9/11] Checking governance graph & UI contract...
  ✓ Graph invariants satisfied
  ✓ UI contract validation PASSED
```

#### Common Issues

**Missing data-uiid in HTML:**
- Add `data-uiid="region.id"` attribute to the appropriate element
- Check that region ID matches exactly (case-sensitive)

**Region kind not defined:**
- Add the kind to `region_kinds` map in `ui_contract.yaml`
- Or use an existing kind like `panel`, `header`, `navigation`

**Duplicate region ID:**
- Region IDs must be unique within each screen
- Use dot notation: `screen.region` for clarity

**Route not reachable:**
- Ensure the route is defined in `app-http` router
- Check for typos in the `route` field

#### Notes

- **Governance artifact**: UI contract is a first-class governed spec
- **Agent-friendly**: `/platform/ui/contract` endpoint enables programmatic access
- **Testing**: DOM tests in `crates/app-http/tests/ui_contract_dom.rs`
- **Part of selftest**: Integrated into step 9 governance check

---

## Command Comparison

| Command | Speed | Coverage | Use Case |
|---------|-------|----------|----------|
| `check` | Fast | Code quality | Every commit |
| `bdd` | Medium | Acceptance | After AC work |
| `ac-status` | Fast | AC coverage | After BDD tests |
| `policy-test` | Fast | Governance | Validate policies |
| `bundle` | Fast | Context gen | Before LLM use |
| `quickstart` | Medium | Basic validation | First run |
| `selftest` | Slow | Comprehensive | CI, releases |
| `idp-check` | Medium | IDP surface | After API changes |
| `contracts-check` | Fast | Doc governance | Before PR merge |
| `contracts-fmt` | Fast | Doc sync | After selftest changes |
| `ui-contract-check` | Medium | UI contract + DOM | After UI changes |

---

## Environment Variables

### RUST_LOG

Controls tracing output for all commands:

```bash
# Default (INFO level)
cargo run -p xtask -- check

# Verbose (DEBUG level)
RUST_LOG=debug cargo run -p xtask -- check

# Specific crate
RUST_LOG=xtask=trace cargo run -p xtask -- check
```

### CARGO_TERM_COLOR

Controls color output:

```bash
# Force colors (in CI)
CARGO_TERM_COLOR=always cargo run -p xtask -- check

# Disable colors
CARGO_TERM_COLOR=never cargo run -p xtask -- check
```

### XTASK_LOW_RESOURCES

Enables low-resource mode for selftest (reduces output, optimizes for CI):

```bash
# Standard mode (verbose output)
cargo run -p xtask -- selftest

# Low-resource mode (condensed output)
XTASK_LOW_RESOURCES=1 cargo run -p xtask -- selftest
```

**When to use:**
- **CI environments** - Reduces log output, easier to parse results
- **Constrained environments** - Lower memory/CPU usage
- **Automated workflows** - Cleaner output for scripting
- **Quick local checks** - Faster feedback without verbose BDD output

**What it does:**
- Suppresses detailed BDD scenario output
- Condenses step-by-step progress messages
- Maintains same validation rigor (all 7 steps run)
- Same exit codes (0 = pass, non-zero = fail)

---

## Tips & Tricks

### Run Multiple Commands

```bash
# Sequential (stops on first failure)
cargo run -p xtask -- check && cargo run -p xtask -- bdd

# Always run both (for CI)
cargo run -p xtask -- check; cargo run -p xtask -- bdd
```

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Re-run on file changes
cargo watch -x 'run -p xtask -- check'
```

### Parallel in CI

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: cargo run -p xtask -- check

  bdd:
    runs-on: ubuntu-latest
    steps:
      - run: cargo run -p xtask -- bdd
```

### Alias for Convenience

Add to `~/.bashrc` or `~/.zshrc`:

```bash
alias xt='cargo run -p xtask --'

# Then use:
xt check
xt bdd
xt bundle implement_ac
```

---

## See Also

- `TEMPLATE_API.md` - Full API specification with schemas
- `docs/how-to/` - Task-oriented guides
- `docs/explanation/architecture.md` - Design rationale

