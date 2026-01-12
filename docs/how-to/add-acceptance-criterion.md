# How-to: Add an Acceptance Criterion

<!-- doclint:disable orphan-version -->
<!-- Contains example gherkin snippets with template version headers -->

This guide walks you through adding a new Acceptance Criterion (AC) to the template, from spec definition through test validation.

**What you'll learn:**

- How to structure ACs in `specs/spec_ledger.yaml`
- How to write BDD scenarios tagged with `@AC-xxx`
- How to map unit/integration tests to ACs
- How to validate and debug AC coverage
- Common pitfalls and how to avoid them

**Prerequisites:**

- Development environment set up (`cargo xtask dev-up`)
- Basic understanding of the spec ledger structure (see `docs/explanation/TEMPLATE-CONTRACTS.md`)
- Familiarity with Gherkin/BDD syntax (see existing `.feature` files)

---

## Overview

Acceptance Criteria (ACs) are the atomic units of verifiable behavior in this template. Each AC:

1. Lives in `specs/spec_ledger.yaml` under a Requirement (REQ)
2. Is tested by one or more BDD scenarios, unit tests, or integration tests
3. Can be validated independently via `cargo xtask test-ac AC-XXX`
4. Contributes to overall governance health shown in `cargo xtask ac-status`

The workflow follows this shape:

```
Story (US-XXX)
  └─ Requirement (REQ-XXX)
      └─ Acceptance Criterion (AC-XXX)
          ├─ BDD scenario (@AC-XXX tag in .feature file)
          ├─ Unit test (optional, for detailed validation)
          └─ Integration test (optional, for end-to-end flows)
```

---

## Step 1: Understand the context

Before adding an AC, identify:

- **Which Story** (US-XXX) describes the user value?
- **Which Requirement** (REQ-XXX) captures the functional need?
- **What specific behavior** are you making testable?

**Example scenario:**

You're adding a feature to list tasks filtered by status. The story `US-TASKS-001` exists, and you have a requirement `REQ-TASKS-QUERY` for "Task Retrieval". Now you need an AC for "Filtering tasks by status returns only matching tasks".

---

## Step 2: Add the AC to spec_ledger.yaml

### Option A: Use the CLI helper (recommended)

```bash
cargo xtask ac-new AC-TASKS-001 "Filtering tasks by status returns only matching tasks" \
  --story US-TASKS-001 \
  --requirement REQ-TASKS-QUERY
```

This scaffolds the AC in the correct location and validates the structure.

### Option B: Manual YAML editing

Open `specs/spec_ledger.yaml` and find your requirement. Add the AC under `acceptance_criteria`:

```yaml
stories:
  - id: US-TASKS-001
    title: "Task Management"
    adr: ADR-0042  # Reference to architectural decision
    requirements:
      - id: REQ-TASKS-QUERY
        title: "Task Retrieval"
        tags: [platform, api]
        must_have_ac: true
        acceptance_criteria:

          # Your new AC goes here
          - id: AC-TASKS-001
            text: "Filtering tasks by status returns only matching tasks"
            tags: [kernel, api]
            must_have_ac: true
            tests:
              - type: bdd
                tag: "@AC-TASKS-001"
                file: "specs/features/tasks.feature"
              - type: unit
                tag: "test_filter_tasks_by_status"
                module: "domain::tasks::tests"
                file: "crates/domain/src/tasks.rs"
```

**Field explanations:**

- **id**: Unique identifier following pattern `AC-<DOMAIN>-<NUMBER>` (e.g., `AC-TASKS-001`, `AC-TPL-007`)
- **text**: Human-readable acceptance criterion (what behavior is expected)
- **tags**: Categories for filtering and reporting
  - `kernel` = must work in every deployment
  - `platform` = core infrastructure feature
  - `api` = external API endpoint
  - `governance` = self-test/validation feature
- **must_have_ac**: `true` means this AC blocks release if failing
- **tests**: Array of test mappings (see Step 3)
- **adr** (optional): Link to Architecture Decision Record if this AC relates to a design choice
- **note** (optional): Context for developers (e.g., "BDD @ci-only to avoid recursive selftest")

---

## Step 3: Map tests to the AC

Each AC must have at least one test. You can combine BDD, unit, and integration tests:

### BDD tests (behavior-driven)

Best for end-to-end user flows and API contracts.

```yaml
tests:
  - type: bdd
    tag: "@AC-TASKS-001"
    file: "specs/features/tasks.feature"
```

**What this means:**

- Scenarios in `specs/features/tasks.feature` tagged with `@AC-TASKS-001` validate this AC
- Run with `cargo xtask test-ac AC-TASKS-001` or `cargo xtask bdd`
- Tag must match exactly (case-sensitive)

### Unit tests (focused logic)

Best for domain logic, parsing, validation, error handling.

```yaml
tests:
  - type: unit
    tag: "test_filter_tasks_by_status"
    module: "domain::tasks::tests"
    file: "crates/domain/src/tasks.rs"
```

**What this means:**

- A Rust test function named `test_filter_tasks_by_status` in `crates/domain/src/tasks.rs`
- Test is in module `domain::tasks::tests` (matches `#[cfg(test)] mod tests { ... }`)
- Run with `cargo test test_filter_tasks_by_status` or `cargo xtask test-ac AC-TASKS-001`

### Integration tests (multi-component flows)

Best for xtask commands, platform APIs, cross-crate interactions.

```yaml
tests:
  - type: integration
    tag: "@AC-TASKS-001"
    file: "specs/features/tasks.feature"
```

**What this means:**

- Similar to BDD but marks it as integration-level in reporting
- Usually runs slower, involves HTTP, filesystem, or process spawning
- Same tagging convention as BDD

### Example: AC with multiple test types

```yaml
- id: AC-PLT-015
  text: "`cargo xtask selftest` enforces devex contract (required commands exist)"
  tags: [kernel, governance]
  must_have_ac: true
  note: "Unit tests verify contract enforcement logic; BDD @ci-only to avoid recursive selftest."
  adr: ADR-0017
  tests:
    # Unit tests for internal logic
    - type: unit
      tag: "devex_contract_enforced_missing_commands"
      module: "commands::selftest::tests::devex_contract_enforced_missing_commands"
      file: "crates/xtask/src/commands/selftest.rs"
    - type: unit
      tag: "devex_contract_enforced_all_present"
      module: "commands::selftest::tests::devex_contract_enforced_all_present"
      file: "crates/xtask/src/commands/selftest.rs"
    - type: unit
      tag: "devex_contract_real_spec_satisfied"
      module: "commands::selftest::tests::devex_contract_real_spec_satisfied"
      file: "crates/xtask/src/commands/selftest.rs"
    # BDD for end-to-end validation
    - type: integration
      tag: "@AC-PLT-015"
      file: "specs/features/xtask_devex.feature"
```

**Why multiple tests?**

- Unit tests validate edge cases (missing commands, partial matches)
- Integration tests validate the actual behavior users experience
- Provides defense-in-depth: if one test layer breaks, others catch regressions

---

## Step 4: Write the BDD scenario

Create or edit a `.feature` file in `specs/features/`. Follow Gherkin syntax and tag scenarios with your AC ID.

### Example: Simple scenario

File: `specs/features/tasks.feature`

```gherkin
# Template Version: v3.3.9
# Schema: spec_ledger.yaml v1.0
# Last Updated: 2025-12-12

Feature: Task Management
  As a developer
  I want to filter tasks by status
  So that I can focus on current work

  @AC-TASKS-001
  Scenario: Filter tasks by Todo status
    Given I have tasks with status "Todo" and "InProgress"
    When I GET /platform/tasks?status=Todo
    Then I receive 200 OK
    And the response contains only tasks with status "Todo"
    And the response does not contain tasks with status "InProgress"
```

### Example: Scenario Outline (multiple test cases)

```gherkin
  @AC-TASKS-001
  Scenario Outline: Filter tasks by different statuses
    Given I have tasks with various statuses
    When I GET /platform/tasks?status=<status>
    Then I receive 200 OK
    And all returned tasks have status "<status>"

    Examples:
      | status     |
      | Todo       |
      | InProgress |
      | Done       |
```

### Example: Multiple scenarios for one AC

```gherkin
  @AC-TPL-004
  Scenario: X-Request-ID is propagated when provided in request
    Given I set "X-Request-ID" header to "test-request-123"
    When I POST /api/echo with invalid data { "message": "" }
    Then I receive a 4xx response
    And the response includes "X-Request-ID" header with value "test-request-123"
    And the "requestId" field in response body equals "test-request-123"

  @AC-TPL-004
  Scenario: X-Request-ID is generated when not provided in request
    When I POST /api/echo with invalid data { "message": "" }
    Then I receive a 4xx response
    And the response includes "X-Request-ID" header
    And the "X-Request-ID" header is a valid UUID or request identifier
```

**BDD best practices:**

- Use clear Given/When/Then structure
- One logical behavior per scenario (avoid cramming multiple assertions)
- Tag every scenario with its AC ID (`@AC-XXX`)
- Add extra tags for filtering (`@smoke`, `@api`, `@ci-only`)
- Keep scenarios readable by non-developers (product/QA can review)
- Use Background for common setup steps

---

## Step 5: Implement code and tests

Now that you have the contract (spec + BDD), implement:

1. **Domain logic** (e.g., `crates/domain/src/tasks.rs`)
2. **Unit tests** in `#[cfg(test)] mod tests { ... }`
3. **HTTP handler** (e.g., `crates/app-http/src/handlers/tasks.rs`)
4. **BDD step definitions** in `crates/acceptance/tests/` if needed

**Example unit test:**

File: `crates/domain/src/tasks.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_tasks_by_status() {
        let tasks = vec![
            Task { id: "1".into(), status: TaskStatus::Todo, ..Default::default() },
            Task { id: "2".into(), status: TaskStatus::InProgress, ..Default::default() },
            Task { id: "3".into(), status: TaskStatus::Todo, ..Default::default() },
        ];

        let filtered = tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .collect::<Vec<_>>();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "3");
    }
}
```

**Pro tip:** Write tests first (TDD), then implement. The AC text tells you exactly what behavior to test.

---

## Step 6: Validate the AC

Use the validation ladder to progressively check your work.

### 6.1: Check syntax and formatting

```bash
# Format Rust code
cargo fmt

# Lint with clippy
cargo clippy

# Validate BDD syntax
cargo xtask bdd --dry-run
```

### 6.2: View tests mapped to your AC

```bash
cargo xtask ac-tests AC-TASKS-001
```

**Expected output:**

```
================================================================================
Acceptance Criterion: AC-TASKS-001
================================================================================

Story: US-TASKS-001
Requirement: REQ-TASKS-QUERY
Text: Filtering tasks by status returns only matching tasks

Mapped Tests:
--------------------------------------------------------------------------------

[1] Type: bdd
    Tag: @AC-TASKS-001
    File: specs/features/tasks.feature

[2] Type: unit
    Tag: test_filter_tasks_by_status
    Module: domain::tasks::tests
    File: crates/domain/src/tasks.rs

Run Tests:
--------------------------------------------------------------------------------

  BDD/Integration: cargo xtask test-ac AC-TASKS-001
  Unit: cargo test test_filter_tasks_by_status
  Direct BDD: CUCUMBER_TAG_EXPRESSION='@AC-TASKS-001' cargo test -p acceptance
```

**What this tells you:**

- Which tests are mapped to this AC
- How to run them individually
- Where to find the test code

### 6.3: Run tests for the AC

```bash
cargo xtask test-ac AC-TASKS-001
```

**Success output:**

```
[INFO] Looking up AC: AC-TASKS-001
[INFO] Found AC: AC-TASKS-001 (requirement: REQ-TASKS-QUERY)

[INFO] Searching for BDD scenarios tagged with @AC-TASKS-001...
[INFO] Found 1 scenario(s):
  - Filter tasks by Todo status (features/tasks.feature)

[INFO] Running acceptance tests for @AC-TASKS-001...

[PASS] All tests passed for AC-TASKS-001
       Scenarios: 1
```

**Failure output:**

```
[INFO] Looking up AC: AC-TASKS-001
[INFO] Found AC: AC-TASKS-001 (requirement: REQ-TASKS-QUERY)

[INFO] Searching for BDD scenarios tagged with @AC-TASKS-001...
[INFO] Found 1 scenario(s):
  - Filter tasks by Todo status (features/tasks.feature)

[INFO] Running acceptance tests for @AC-TASKS-001...

[FAIL] Tests failed for AC-TASKS-001

Feature: Task Management
  Scenario: Filter tasks by Todo status
    Given I have tasks with status "Todo" and "InProgress"
    When I GET /platform/tasks?status=Todo
    Then I receive 200 OK ✓
    And the response contains only tasks with status "Todo" ✗
      Expected 2 tasks with status "Todo", got 3

1 scenario (1 failed)
5 steps (1 failed, 4 passed)

[ERROR] AC-TASKS-001 validation failed
```

**What to do on failure:**

1. Read the error message carefully (which step failed, expected vs. actual)
2. Run the BDD test directly to see detailed output:
   ```bash
   CUCUMBER_TAG_EXPRESSION='@AC-TASKS-001' cargo test -p acceptance -- --nocapture
   ```
3. Debug the implementation (add `println!`, use a debugger)
4. Fix the code, re-run `cargo xtask test-ac AC-TASKS-001`

### 6.4: Check overall AC coverage

```bash
cargo xtask ac-status
```

**Output:**

```
Parsing ledger: specs/spec_ledger.yaml
  Found 134 ACs
Parsing AC coverage (primary path): target/ac/coverage.jsonl
  Found 1 scenarios
  Found results for 1 ACs
Running unit tests for AC mappings...
  Captured results for 410 unit tests
Generating status: docs/feature_status.md

[OK] All ACs passed
```

**Interpreting the report:**

- `docs/feature_status.md` is regenerated with pass/fail for each AC
- Green checkmarks (✅) = AC passing
- Red crosses (❌) = AC failing or missing tests
- Review `docs/feature_status.md` to see detailed breakdown by Story/Requirement

### 6.5: Run the full selftest

```bash
cargo xtask selftest
```

This runs all 11 governance checks:

1. Rust fmt + clippy
2. Unit tests
3. BDD tests
4. Spec ledger validation
5. DevEx contract enforcement
6. Skills governance
7. Agents governance
8. ADR linkage
9. Documentation checks
10. Spellcheck
11. License + SBOM

**When to run selftest:**

- Before opening a pull request
- After significant changes to specs, tests, or code
- When `cargo xtask ac-status` shows failures you can't explain

---

## Step 7: Iterate and refine

Your AC might need adjustments as you learn more about the behavior.

**Refinement checklist:**

- ✅ AC text is specific and testable (not vague like "works correctly")
- ✅ BDD scenarios cover happy path and edge cases
- ✅ Unit tests validate domain logic in isolation
- ✅ Tags are accurate (`kernel`, `platform`, `api`, `governance`)
- ✅ `must_have_ac: true` if this blocks release (vs. nice-to-have)
- ✅ All tests pass: `cargo xtask test-ac AC-XXX` is green
- ✅ ADR linked if this AC relates to an architectural decision
- ✅ Docs updated if this changes user-facing behavior

**When to split an AC:**

If your AC requires 5+ BDD scenarios or 10+ unit tests, consider splitting into multiple ACs. Each AC should be independently verifiable and releasable.

**Example refactor:**

Before (too broad):

```yaml
- id: AC-TASKS-001
  text: "Task filtering and sorting works correctly"
```

After (split into focused ACs):

```yaml
- id: AC-TASKS-001
  text: "Filtering tasks by status returns only matching tasks"

- id: AC-TASKS-002
  text: "Sorting tasks by created_at in ascending order returns oldest first"

- id: AC-TASKS-003
  text: "Combining status filter and created_at sort applies both criteria"
```

---

## Common pitfalls and debugging

### Pitfall 1: AC tag mismatch between YAML and .feature file

**Symptom:**

```
[WARN] No scenarios found for @AC-TASKS-001
[FAIL] AC has no test coverage
```

**Fix:**

- Check `specs/features/tasks.feature` has `@AC-TASKS-001` tag (exact match, case-sensitive)
- Ensure tag is on the `Scenario:` line, not inside the steps
- Run `grep -r "@AC-TASKS-001" specs/features/` to verify

### Pitfall 2: Unit test not discovered

**Symptom:**

```
[INFO] Mapped unit tests: 0
[WARN] AC-TASKS-001 has no unit tests
```

**Fix:**

- Ensure test function name matches `tag` in YAML exactly
- Test must be in `#[cfg(test)] mod tests { ... }` (or another `#[cfg(test)]` module)
- File path in YAML must match actual file location
- Run `cargo test test_filter_tasks_by_status -- --list` to verify test exists

### Pitfall 3: BDD scenario fails with "step not implemented"

**Symptom:**

```
[ERROR] Step not implemented: "I have tasks with status "Todo" and "InProgress""
```

**Fix:**

- Check `crates/acceptance/tests/steps/` for step definitions
- Add missing step implementation following existing patterns
- Common pattern: `#[given(regex = r"^I have tasks with status (.+)$")]`
- See existing step files for examples (tasks.rs, http.rs, etc.)

### Pitfall 4: AC passing locally but failing in CI

**Symptom:**

- `cargo xtask test-ac AC-XXX` passes on your machine
- CI fails with "AC-XXX validation failed"

**Possible causes:**

1. **Environment differences**: CI may not have Docker, or different Rust version
   - Fix: Check `.github/workflows/` for CI environment
   - Use `cargo xtask doctor` to compare environments
2. **Timing/race conditions**: BDD tests may depend on timing assumptions
   - Fix: Add explicit waits, avoid sleep() hacks
3. **Hardcoded paths**: Tests reference `/home/you/...` instead of repo-relative paths
   - Fix: Use `env::current_dir()` or test fixtures
4. **Missing test data**: Local fixtures not committed to git
   - Fix: Ensure test data is in `specs/fixtures/` or generated in test setup

**Debugging CI failures:**

```bash
# Reproduce CI environment locally (if using Nix)
nix develop

# Run exactly what CI runs
cargo xtask selftest

# Check for git-ignored files that tests depend on
git status --ignored
```

### Pitfall 5: Circular dependencies in spec_ledger.yaml

**Symptom:**

```
[ERROR] Spec ledger validation failed: AC-TASKS-001 references REQ-TASKS-002, but REQ-TASKS-002 does not exist
```

**Fix:**

- Ensure `id` fields are unique across all stories, requirements, and ACs
- Check that AC is nested under the correct requirement
- Validate YAML syntax with `cargo xtask selftest` (runs schema validation)

### Pitfall 6: AC marked must_have_ac: true but has no tests

**Symptom:**

```
[ERROR] AC-TASKS-001 is marked must_have_ac: true but has no passing tests
[FAIL] Selftest blocked by missing critical AC coverage
```

**Fix:**

- Add at least one test (BDD, unit, or integration) to the `tests:` array
- Ensure test actually runs and passes
- If AC is aspirational (not yet implemented), set `must_have_ac: false` and add a note:
  ```yaml
  must_have_ac: false
  note: "Planned for v2.0; not blocking current release"
  ```

### Pitfall 7: BDD scenario too brittle (breaks on unrelated changes)

**Symptom:**

- Small refactors (renaming fields, changing log format) break many scenarios
- Tests couple to implementation details instead of behavior

**Fix:**

- Focus BDD on user-observable behavior, not internal mechanics
- Avoid asserting exact JSON structure; check for required fields only
- Use regex or partial matches instead of exact string matches
- Example: Instead of `And the response equals '{"id":"1","status":"Todo"}'`, use `And the response contains "id" and "status"`

### Pitfall 8: Forgetting to link ADR to AC

**Symptom:**

```
[WARN] AC-TASKS-001 has no linked ADR, but makes architectural assumptions
```

**Fix:**

- If your AC relates to a design decision (e.g., "use SQLite", "async-first"), create or link an ADR:
  ```yaml
  - id: AC-TASKS-001
    text: "Task storage uses SQLite with WAL mode for concurrency"
    adr: ADR-0042  # Link to docs/adr/ADR-0042-task-storage-sqlite.md
  ```
- Run `cargo xtask adr-check` to validate all ADR links resolve

---

## Quick reference

### YAML structure

```yaml
stories:
  - id: US-DOMAIN-001
    title: "User-facing feature name"
    adr: ADR-XXXX  # Optional: architectural decision
    requirements:
      - id: REQ-DOMAIN-TOPIC
        title: "Specific functional requirement"
        tags: [platform, api]
        must_have_ac: true
        acceptance_criteria:
          - id: AC-DOMAIN-001
            text: "Specific testable behavior"
            tags: [kernel, api]
            must_have_ac: true
            adr: ADR-XXXX  # Optional: design decision
            note: "Optional context for developers"
            tests:
              - type: bdd
                tag: "@AC-DOMAIN-001"
                file: "specs/features/domain.feature"
              - type: unit
                tag: "test_function_name"
                module: "crate::module::tests"
                file: "crates/crate/src/module.rs"
```

### BDD scenario template

```gherkin
# Template Version: v3.3.9
# Schema: spec_ledger.yaml v1.0
# Last Updated: YYYY-MM-DD

Feature: Human-readable feature name
  As a [role]
  I want [capability]
  So that [benefit]

  Background:
    Given common setup step

  @AC-DOMAIN-001
  Scenario: Specific behavior description
    Given precondition
    When action
    Then expected outcome
    And additional assertion
```

### Command cheat sheet

```bash
# Create new AC (scaffolds YAML)
cargo xtask ac-new AC-XXX "Description" --story US-XXX --requirement REQ-XXX

# View tests mapped to AC
cargo xtask ac-tests AC-XXX

# Run tests for specific AC
cargo xtask test-ac AC-XXX

# Check all AC coverage
cargo xtask ac-status

# Validate BDD syntax
cargo xtask bdd --dry-run

# Run all BDD tests
cargo xtask bdd

# Full governance check
cargo xtask selftest

# Run only unit tests
cargo test

# Run specific unit test
cargo test test_function_name

# Run BDD for specific tag
CUCUMBER_TAG_EXPRESSION='@AC-XXX' cargo test -p acceptance
```

---

## Next steps

- **Explore existing ACs**: Read `specs/spec_ledger.yaml` to see patterns
- **Review feature files**: Check `specs/features/*.feature` for BDD examples
- **Understand the validation ladder**: See `docs/explanation/TEMPLATE-CONTRACTS.md`
- **Learn about selftest**: Read `docs/how-to/understand-selftest.md` (if it exists)
- **Contribute to governance**: Help improve this process by filing issues or ADRs

**Questions or issues?**

- File a GitHub issue with label `governance` or `documentation`
- Check `FRICTION_LOG.md` for known DevEx pain points
- Consult `docs/AGENT_GUIDE.md` for AI-assisted AC development

