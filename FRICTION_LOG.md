# Friction Log

This log captures friction points discovered during developen http://localhost:3000/ui/graphge.

## AGENT-001: UI/API Inconsistency - Tasks Not Shown in UI/Hints When tasks_state.yaml Missing

**Date:** 2025-11-20
**Reporter:** Agent verification run
**Root Cause:**  
- `/platform/tasks` reads directly from `tasks.yaml` using `spec_runtime::load_tasks()` (works)
- `/ui/tasks` and `/platform/agent/hints` use `TaskService.list_tasks()` which calls `GovernanceRepository.find_all_tasks()`
- `FsGovernanceRepository.find_all_tasks()` only returned tasks from `tasks_state.yaml`
- When `tasks_state.yaml` doesn't exist, `get_all_tasks()` returned an empty HashMap

**Expected Behavior:**  
`FsGovernanceRepository.find_all_tasks()` should merge:
- Task definitions (id, title) from `tasks.yaml`
- Task status from `tasks_state.yaml` (or default to status field in tasks.yaml if no state file)

**Impact:**  
- UI is unusable - shows no tasks
- Agent hints API returns no work - agent can't discover tasks
- Data correctness verification incomplete

**Fix Implemented:**  
- Updated `FsGovernanceRepository.find_all_tasks()` to merge definitions and status correctly.
- Added tests verifying UI and hints now show tasks regardless of `tasks_state.yaml` presence.

**Verification:**  
- `/ui/tasks` shows all tasks from `tasks.yaml`
- `/platform/agent/hints` returns tasks with Todo/InProgress status
- Both work whether or not `tasks_state.yaml` exists

**Status:** Resolved (2025‑11‑20)num

**Verification:**  
After fix:
- `/ui/tasks` should show all tasks from tasks.yaml
- `/platform/agent/hints` should return tasks with Todo/InProgress status
- Both should work whether or not tasks_state.yaml exists

---

### AGENT-001: Port discovery focurl http://localhost:3000/healthopment
**Discovered**: 2025-11-20T22:42
**Severity**: Low
**Context**: Agent pilot run - Phase 1 (Discover Work)

**What happened**:
When attempting to call `GET /platform/agent/hints`, initially tried port 8080 (common default) but got connection refused. Had to use `lsof` to discover app-http was running on port 3000.

**Expected behavior**:
Either:
1. Agent documentation should specify the default port (3000)
2. `cargo xtask` should have a command to show running service ports
3. App should log the URL it's listening on at startup

**Workaround**:
```bash
lsof -i | grep app-http
# OR
ps aux | grep app-http | grep -v grep, then lsof -p <PID>
```
**Status**: Open
