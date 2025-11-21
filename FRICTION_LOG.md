# Friction Log

This log captures friction points discovered during development or pilot usage.

## AGENT-001: UI/API Inconsistency - Tasks Not Shown in UI/Hints When tasks_state.yaml Missing

**Date:** 2025-11-20
**Reporter:** Agent verification run
**Severity:** High

**Problem:**  
The `/platform/tasks` JSON API shows tasks correctly from `tasks.yaml`, but:
- `/ui/tasks` (kanban board) shows empty columns
- `/platform/agent/hints` returns empty `next_tasks` array

**Root Cause:**  
- `/platform/tasks` reads directly from `tasks.yaml` using `spec_runtime::load_tasks()` (works)
- `/ui/tasks` and `/platform/agent/hints` use `TaskService.list_tasks()` which calls `GovernanceRepository.find_all_tasks()`
- `FsGovernanceRepository.find_all_tasks()` only returns tasks from `tasks_state.yaml`
- When `tasks_state.yaml` doesn't exist, `get_all_tasks()` returns an empty HashMap

**Expected Behavior:**  
`FsGovernanceRepository.find_all_tasks()` should merge:
- Task definitions (id, title) from `tasks.yaml`
- Task status from `tasks_state.yaml` (or default to status field in tasks.yaml if no state file)

**Impact:**  
- UI is unusable - shows no tasks
- Agent hints API returns no work - agent can't discover tasks
- Data correctness verification incomplete

**Fix Required:**  
Update `FsGovernanceRepository.find_all_tasks()` in `adapters-spec-fs/src/lib.rs` to:
1. Load all task definitions from `tasks.yaml`
2. Load status overrides from `tasks_state.yaml` (if exists)
3. Merge them together, using status from state file if available, otherwise from definition
4. Parse status strings from tasks.yaml to TaskStatus enum

**Verification:**  
After fix:
- `/ui/tasks` should show all tasks from tasks.yaml
- `/platform/agent/hints` should return tasks with Todo/InProgress status
- Both should work whether or not tasks_state.yaml exists
