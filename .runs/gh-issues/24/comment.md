## Investigation Report: Issue #24 - Missing xtask Commands

### Status
**Status:** PARTIALLY IMPLEMENTED
**Local gates:** Command audit completed

### Evidence

**Current commands:** 81 implemented

**Commands from issue:**
| Command | Status |
|---------|--------|
| `profile-build` | MISSING |
| `ac-search` | MISSING |
| `health-check` | MISSING (partial via doctor+ac-status) |
| `suggest-next` | ✅ EXISTS |
| `friction-new --interactive` | PARTIAL (non-interactive exists) |
| `task-graph` | MISSING |

**Actually missing:** 4 commands (not 6)

### Plan

**Implementation order:**
1. `ac-search` (1h) - High DevX impact
2. `profile-build` (2h) - CI useful
3. `health-check` (2h) - Unifies existing commands
4. `task-graph` (1.5h) - Visualization aid

**Total:** ~7.5 hours

### Decision / Next Action

**Recommend:** Keep open, update to reflect `suggest-next` already exists. 4 commands remain.
