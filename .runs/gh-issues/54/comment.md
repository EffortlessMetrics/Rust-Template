## Investigation Report: Issue #54 - TODO in docs_check.rs

### Status
**Status:** ALREADY RESOLVED ✅
**Local gates:** Code audit completed

### Evidence

The TODO about promoting kernel REQ doc coverage from soft to hard check has been addressed:

**Current state** (`crates/xtask/src/commands/docs_check.rs:138-157`):
- Line 140: Comment now states "This is now a hard check since kernel documentation coverage is complete"
- Lines 111-114: `anyhow::bail!()` called if kernel REQs lack documentation
- The check increments `issues` counter and causes exit code 1

**Current blocking issue:**
- `REQ-PLT-ISSUES-ENDPOINT` missing documentation (from recent #74 merge)
- This is a separate tracking item, not related to the TODO

### Decision / Next Action

**Recommend:** CLOSE AS RESOLVED - The TODO has been addressed. The kernel REQ doc coverage check is now hard-enforced. The remaining `REQ-PLT-ISSUES-ENDPOINT` documentation gap is a separate issue.
