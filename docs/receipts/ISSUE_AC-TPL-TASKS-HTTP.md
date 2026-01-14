<!-- doclint:disable orphan-version -->
# Issue: AC-TPL-TASKS-HTTP - Task Status Normalization Bug

**Date:** 2025-12-03
**Status:** Known Issue (deferred to future release)
**Related ACs:** AC-TPL-TASKS-HTTP
**Priority:** Low (non-kernel AC, `must_have_ac: false`, tagged `[future]`)

## Problem

The task status normalization function does not properly normalize `closed` to `Done`.

**Expected behavior:**
| Input status | Normalized output |
|--------------|-------------------|
| `open`       | `Todo`            |
| `todo`       | `Todo`            |
| `in_progress`| `InProgress`      |
| `in-progress`| `InProgress`      |
| `closed`     | `Done`            |
| `done`       | `Done`            |

**Actual behavior:**
- `closed` normalizes to `Todo` instead of `Done`

## Affected Scenario

From `specs/features/platform_tasks.feature`:

```gherkin
@AC-TPL-TASKS-HTTP @wip
# NOTE: The @wip tag is intentional - this scenario is deferred because:
# 1. The task status normalization function needs to be updated to handle "closed" → "Done"
# 2. This is a non-kernel AC (must_have_ac: false) that doesn't block selftest
# 3. The fix requires updating the status normalization logic in the HTTP adapter
# 4. See "Fix" section below for implementation steps
Scenario: Task statuses are normalized to the canonical set
  Given the following tasks exist in "specs/tasks.yaml":
    | id              | title                     | status        | requirement      |
    | TASK-NORM-001   | Backlog item              | open          | REQ-TPL-HEALTH  |
    | TASK-NORM-002   | Work in progress (snake)  | in_progress   | REQ-TPL-HEALTH  |
    | TASK-NORM-003   | Work in progress (dash)   | in-progress   | REQ-TPL-HEALTH  |
    | TASK-NORM-004   | Closed alias              | closed        | REQ-TPL-HEALTH  |
  When I send a GET request to "/platform/tasks"
  Then the response status code should be 200
  And the response body should contain '"status":"Todo"'
  And the response body should contain '"status":"InProgress"'
  And the response body should contain '"status":"Done"'
```

## Root Cause

The status normalization function is likely missing the `closed` → `Done` mapping. Check:

- `crates/model/src/task.rs` or similar model file
- Any `normalize_status()` or equivalent function
- The `/platform/tasks` HTTP handler

## Fix

1. Locate the status normalization function
2. Add `"closed" => TaskStatus::Done` to the mapping
3. Remove `@wip` tag from the scenario
4. Run `cargo xtask test-ac AC-TPL-TASKS-HTTP` to verify

## Impact

This is a **non-kernel** AC (`must_have_ac: false`, tagged `[future]`), so it does not block selftest. The scenario is `@wip` to keep the test suite green while allowing incremental progress.

### Why This Is Deferred

1. **Low Priority**: The `closed` status alias is not commonly used in production scenarios
2. **Non-Blocking**: The canonical `Done` status works correctly; this is an edge case for compatibility
3. **Resource Allocation**: Focus is on kernel ACs and higher-priority platform features
4. **Low Risk**: The bug doesn't affect core functionality or data integrity

### When To Implement

Consider implementing this fix when:
- User feedback indicates `closed` status is needed for integrations
- A comprehensive status alias audit is performed
- The platform tasks API is refactored for other reasons
- Capacity allows for low-priority non-kernel AC work

## Verification

```bash
# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-TPL-TASKS-HTTP" cargo test -p acceptance --test acceptance

# Check AC status
cargo xtask ac-status | grep AC-TPL-TASKS-HTTP
```
