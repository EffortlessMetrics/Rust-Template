<!-- doclint:disable orphan-version -->
# Issue: AC-TPL-TASKS-HTTP - Task Status Normalization Bug

**Date:** 2025-12-03
**Status:** WIP (scenario parked)
**Related ACs:** AC-TPL-TASKS-HTTP
**Priority:** Low (non-kernel AC, `must_have_ac: false`)

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
# FIXME: "closed" status not normalizing to "Done" - see task status normalization code
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

## Verification

```bash
# Run targeted BDD tests
CUCUMBER_TAG_EXPRESSION="@AC-TPL-TASKS-HTTP" cargo test -p acceptance --test acceptance

# Check AC status
cargo xtask ac-status | grep AC-TPL-TASKS-HTTP
```
