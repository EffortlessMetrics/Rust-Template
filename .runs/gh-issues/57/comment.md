## Investigation Report: Issue #57 - Replace panic!() with assert!()

### Status
**Status:** fix-ready - 18 panic!() calls identified
**Local gates:** Code audit completed

### Evidence

**Total panic!() calls: 18** (reduced from 70+ mentioned in issue)

**Distribution:**
- `governance_tasks.rs`: 12 calls (lines 213, 286, 541, 565, 589, 617, 814, 883, 886, 984, 1007, 1036)
- `agent_hints.rs`: 3 calls (lines 124, 168, 197)
- `xtask_devex.rs`: 3 calls (lines 2982, 3015, 3025)

**Patterns found:**
1. "response should exist" - 6 instances → `assert!(world.last_response.is_some(), "...")`
2. `unwrap_or_else(|| panic!(...))` - 3 instances → use `expect()`
3. Field type validation - 4 instances → use `assert!()`

### Plan

**Phase 1:** Replace "response should exist" with `assert!`
**Phase 2:** Replace `unwrap_or_else` with `expect()`
**Phase 3:** Replace field validation with `assert!()`
**Phase 4:** Consider `#[track_caller]` for helper functions

**Test plan:**
```bash
cargo test -p acceptance
cargo xtask selftest
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Good first issue - concentrated in 3 files, ~2 hours work.
