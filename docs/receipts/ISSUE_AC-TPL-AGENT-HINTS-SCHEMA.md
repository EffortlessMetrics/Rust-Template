# Fix AC-TPL-AGENT-HINTS-SCHEMA: hints schema validation

## AC Definition

From `specs/spec_ledger.yaml` (lines 1132-1158):

```yaml
AC-TPL-AGENT-HINTS-SCHEMA:
  text: |
    Hints returned from `/platform/agent/hints` MUST be objects with fields:
    - `id`: stable hint identifier (string)
    - `kind`: category of hint (e.g., `task`, `governance`, `policy`)
    - `priority`: one of `low`, `medium`, `high`
    - `status`: one of `open`, `in_progress`, `done`
    - `reason`: object with `code` and `details`
    - `target`: object describing what the hint is about
    - `tags`: array of strings (labels)
    - `links`: object with optional `spec`, `task`, `docs`, `adrs`, and `extra`
  requirement: REQ-TPL-AGENT-HINTS
  tags: [kernel, template]
  must_have_ac: true
  tests:
    - { type: integration, tag: "@AC-TPL-AGENT-HINTS-SCHEMA", file: "specs/features/agent_hints.feature" }
```

## Current Status

**Implementation: 100% complete, BDD test failing due to test infrastructure bug**

### ✅ What Works

- ✅ Schema correctly defined in `crates/spec-runtime/src/hints.rs`
- ✅ All 8 required fields present: id, kind, priority, status, reason, target, tags, links
- ✅ Proper enum serialization (snake_case: low, medium, high, open, in_progress, done)
- ✅ HTTP endpoint `/platform/agent/hints` returns AgentHint with full schema
- ✅ **Unit test PASSES**: `hint_serialization_schema` validates all fields
- ✅ Backward compatibility: AgentHint includes convenience fields (task_id, title, etc.)

### ❌ What's Failing (Test Infrastructure Bug)

**BDD test scenario fails because hints array is empty**

**Root Cause:** Test isolation issue in BDD infrastructure

1. BDD step creates tasks in **temporary test directory** (`temp_dir/specs/tasks.yaml`)
2. HTTP app reads tasks from **workspace root** (`resolve_workspace_root()` → canonical location)
3. Result: Endpoint reads empty/original tasks, not the test-created tasks
4. No hints returned → BDD assertion fails

## BDD Test Scenario

From `specs/features/agent_hints.feature` (lines 282-297):

**Scenario: "HTTP hints include all required schema fields"**

```gherkin
@AC-TPL-AGENT-HINTS-SCHEMA
Scenario: HTTP hints include all required schema fields
  Given the following tasks exist in "specs/tasks.yaml":
    | task_id           | status | title              |
    | TASK-SCHEMA-001   | Todo   | Test Schema Fields |
  When I GET "/platform/agent/hints"
  Then the response status should be 200
  And the response should have field "hints"
  And the first hint should have field "id"
  And the first hint should have field "kind"
  And the first hint should have field "priority"
  And the first hint should have field "status"
  And the first hint should have field "reason"
  And the first hint should have field "target"
  And the first hint should have field "tags"
  And the first hint should have field "links"
```

**Current failure:** "hints" array is empty, so assertions fail

## Files Requiring Changes

### Priority 1 (Required)

1. **`crates/acceptance/src/world.rs`** (lines 143-147)

   **Problem:** App is created with workspace root, not temp directory

   **Current code:**

   ```rust
   app: app_http::app(governance_repo),
   ```

   **Fix:**

   ```rust
   app: app_http::app_with_workspace_root(
       governance_repo,
       temp_dir.path().to_path_buf(),
   ),
   ```

2. **`crates/acceptance/src/steps/governance_tasks.rs`** (after line 145)

   **Problem:** App doesn't reload after tasks are created

   **Fix:** Add after line 145:

   ```rust
   // Reload the app to reflect task changes
   world.reload_app();
   ```

### No Implementation Changes Required

- ✅ `crates/spec-runtime/src/hints.rs` - Schema complete
- ✅ `crates/app-http/src/agent.rs` - Endpoint working
- ✅ `specs/features/agent_hints.feature` - Scenario correct

## Root Cause Summary

This is **not a schema design issue**. The schema is correctly implemented and validated by the passing unit test. The BDD test fails because:

1. Test creates tasks in isolated temp directory
2. In-process HTTP app doesn't read from that temp directory
3. App needs to use temp directory as workspace root during tests
4. App needs to reload after test setup modifies specs

## Verification Commands

```bash
# After fixes:
cargo xtask bdd --tag @AC-TPL-AGENT-HINTS-SCHEMA

# Verify all agent hints scenarios
cargo xtask bdd --tag @AC-TPL-AGENT-HINTS

# Verify AC status
cargo xtask ac-status | grep AC-TPL-AGENT-HINTS-SCHEMA

# Full governance check
cargo xtask selftest
```

## Acceptance Criteria

- [ ] Modify world.rs to use `app_with_workspace_root(temp_dir)`
- [ ] Add `world.reload_app()` call after task creation in governance_tasks.rs
- [ ] BDD scenario passes (all 8 schema fields present in hints)
- [ ] `cargo xtask ac-status` shows AC-TPL-AGENT-HINTS-SCHEMA as passing
- [ ] `cargo xtask selftest` passes AC-TPL-AGENT-HINTS-SCHEMA

## Related Files

- Test Infrastructure: `crates/acceptance/src/world.rs:143-147` (needs fix)
- Test Steps: `crates/acceptance/src/steps/governance_tasks.rs:145` (needs reload call)
- Schema Definition: `crates/spec-runtime/src/hints.rs:1-84` (complete ✅)
- Endpoint: `crates/app-http/src/agent.rs:18-210` (complete ✅)
- BDD Scenario: `specs/features/agent_hints.feature:282-297`
- Spec: `specs/spec_ledger.yaml:1132-1158`
- Unit Test: `crates/spec-runtime/src/hints.rs:437-494` (PASSING ✅)

## Estimated Effort

**10 minutes** - Two simple infrastructure fixes

## Labels

`kernel`, `ac-fail`, `must-fix`, `test-infrastructure`, `bdd`
