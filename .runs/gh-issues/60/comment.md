## Investigation Report: Issue #60 - brownfield-demo Mutex Pattern

### Status
**Status:** triaged - LOW priority (example code)
**Local gates:** Code audit completed

### Evidence

**File:** `examples/brownfield-demo/server/src/main.rs`

**3 instances (lines 68, 77, 91):**
```rust
state.lock().expect("items mutex poisoned - a thread panicked while holding the lock");
```

**Analysis:**
- Uses `std::sync::Mutex` in async Tokio context
- Already uses `.expect()` with descriptive message (better than `.unwrap()`)
- Lock duration is short (clone only)
- **Risk:** Minimal - executor blocking unlikely with short locks

### Impact

| Aspect | Assessment |
|--------|------------|
| Executor blocking | Minimal (short locks) |
| Poisoning risk | Low |
| Educational impact | HIGH (users copy examples) |

**Relationship to #16:** Related but different - #16 is CRITICAL production code, #60 is LOW priority example code.

### Plan

**Option 1 (Short-term):** Add educational comments explaining trade-offs
**Option 2 (Medium-term):** Consider `parking_lot::RwLock` migration

### Decision / Next Action

**Recommend:** Keep open as **LOW priority**. Add documentation comments explaining:
- Why pattern is acceptable for demo
- Alternatives for production (`tokio::sync::Mutex`, `parking_lot::RwLock`)
- Link to #16 for production patterns
