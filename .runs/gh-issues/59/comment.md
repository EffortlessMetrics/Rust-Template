## Investigation Report: Issue #59 - Unsafe Env Vars in Tests

### Status
**Status:** CONFIRMED - Overlaps significantly with #13
**Local gates:** Code audit, pattern analysis

### Evidence

**Total unsafe env var manipulations: 61 instances across 14 files**

**By Crate:**

| Crate | Instances | Synchronization |
|-------|-----------|-----------------|
| xtask (kernel.rs, env.rs) | 15+ | ✅ Mutex serialization |
| xtask (ci_local.rs, selftest.rs, tasks.rs) | 15+ | ✗ None |
| acceptance (world.rs, steps/) | 15+ | ✗ None |
| app-http (security_middleware.rs) | 10+ | ✅ EnvVarGuard RAII |

**Patterns Identified:**

1. **RAII + Async Mutex** (app-http) - SAFEST ✅
   - Snapshots and restores values
   - Async-aware locking
   - Automatic cleanup via Drop

2. **Mutex Serialization** (xtask/kernel.rs) - SAFER ✅
   - Global mutex prevents concurrent access
   - All variables restored under lock

3. **Unsafe without synchronization** (multiple) - RISKY ✗
   - No locking mechanism
   - Risk of incomplete restoration

**Critical Issues:**
- `cargo test -- --test-threads=N` with N>1 causes data races
- Some operations have consecutive unsafe blocks without atomic restoration
- Child processes inherit potentially-corrupted env vars

### Impact

- **Test Flakiness:** Tests pass locally (single-threaded), fail in CI (parallel)
- **Token Exposure:** Credentials could leak between tests
- **Blast Radius:** 61 instances across the codebase

### Plan

**This issue overlaps with #13.** Recommend merging efforts.

**Unified fix:**
1. Add `#[serial]` to all env-mutating tests
2. Create `EnvVarGuard` RAII pattern for all test modules
3. Add governance check to prevent regression

**Test plan:**
```bash
cargo test -- --test-threads=4  # Verify parallel safety
cargo xtask selftest
```

### Decision / Next Action

**Recommend:** Consider closing as DUPLICATE of #13, or merge both into unified "Test Environment Safety" epic. Same root cause, same fix.

**Related:** #13 (Unsafe Env Var Manipulation)
