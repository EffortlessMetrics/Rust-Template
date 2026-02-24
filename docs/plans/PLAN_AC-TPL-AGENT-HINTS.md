# Plan: AC-TPL-AGENT-HINTS-SCHEMA - Agent Hints Endpoint Schema

**Date:** 2025-12-02
**Status:** Ready for Implementation
**Related ACs:** AC-TPL-AGENT-HINTS-SCHEMA

## Scope

**Files in scope:**
- `crates/acceptance/src/world.rs` (lines 143-147) - Fix app initialization to use temp directory
- `crates/acceptance/src/steps/governance_tasks.rs` (after line 145) - Add app reload after task creation

**What's already working:**
- ✅ Schema correctly defined in `crates/spec-runtime/src/hints.rs`
- ✅ All 8 required fields present: id, kind, priority, status, reason, target, tags, links
- ✅ Proper enum serialization (snake_case)
- ✅ HTTP endpoint `/platform/agent/hints` returns correct schema
- ✅ Unit test PASSES: `hint_serialization_schema` validates all fields
- ✅ Backward compatibility maintained

**What's broken:**
- ❌ BDD test fails because hints array is empty (test infrastructure bug)

## Goals

1. Fix BDD test infrastructure so HTTP app reads from test-created tasks
2. Make `/platform/agent/hints` BDD tests pass without changing schema implementation
3. Achieve 100% AC-TPL-AGENT-HINTS-SCHEMA coverage

## Root Cause Analysis

**Problem:** Test isolation issue in BDD infrastructure

1. BDD step creates tasks in **temporary test directory** (`temp_dir/specs/tasks.yaml`)
2. HTTP app reads tasks from **workspace root** (`resolve_workspace_root()` → canonical location)
3. Result: Endpoint reads empty/original tasks, not the test-created tasks
4. No hints returned → BDD assertion fails on empty array

**Solution:** Make HTTP app use temp directory as workspace root during tests

## Implementation Steps

1. **Fix app initialization in world.rs** (lines 143-147)

   **Current code:**

   ```rust
   app: app_http::app(governance_repo),
   ```

   **Change to:**

   ```rust
   app: app_http::app_with_workspace_root(
       governance_repo,
       temp_dir.path().to_path_buf(),
   ),
   ```

   **Why:** This tells the app to read specs from the test's temp directory instead of the canonical workspace root

2. **Add app reload after task creation** (`crates/acceptance/src/steps/governance_tasks.rs` after line 145)

   **Add this code:**

   ```rust
   // Reload the app to reflect task changes
   world.reload_app();
   ```

   **Why:** After BDD steps modify specs/tasks.yaml, the in-process HTTP app needs to reload its governance state

3. **Verify no implementation changes needed**
   - Schema is already correct (unit test passes)
   - Endpoint is already working (returns correct structure)
   - Only test infrastructure needs fixing

## Verification Commands

```bash
# Run targeted BDD test
CUCUMBER_TAG_EXPRESSION="@AC-TPL-AGENT-HINTS-SCHEMA" cargo test -p acceptance --test acceptance

# Run all agent hints scenarios
CUCUMBER_TAG_EXPRESSION="@AC-TPL-AGENT-HINTS" cargo test -p acceptance --test acceptance

# Verify unit test still passes
cargo test -p rust-as-spec-runtime hint_serialization_schema

# Verify AC status
cargo xtask ac-status | grep AC-TPL-AGENT-HINTS-SCHEMA

# Full validation
cargo xtask selftest
```

## Definition of Done

- [ ] Modified `world.rs` to use `app_with_workspace_root(temp_dir)`
- [ ] Added `world.reload_app()` call after task creation in `governance_tasks.rs`
- [ ] BDD scenario "@AC-TPL-AGENT-HINTS-SCHEMA" passes
- [ ] All 8 schema fields present in hints (id, kind, priority, status, reason, target, tags, links)
- [ ] Hints array is non-empty in test (contains test-created tasks)
- [ ] Unit test `hint_serialization_schema` still passes
- [ ] `cargo xtask ac-status` shows AC-TPL-AGENT-HINTS-SCHEMA as PASS
- [ ] No other ACs flip to FAIL
- [ ] Test infrastructure changes do not break other BDD scenarios

## Notes

- **Estimated Effort:** 10 minutes - Two simple infrastructure fixes
- **Risk Level:** Low - Changes are isolated to test infrastructure
- **No Schema Changes:** This is purely a test infrastructure fix; the schema implementation is already correct
- **Backward Compatibility:** Maintained - no breaking changes to public APIs
