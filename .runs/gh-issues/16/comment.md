## Investigation Report: Issue #16 - std::sync::Mutex in Async Code

### Status
**Status:** analysis-complete / **Severity:** low-to-moderate
**Local gates:** `cargo xtask selftest` ✓, `cargo xtask check` ✓

### Evidence

**Safe patterns found (6/7):**

| File | Line | Pattern | Status |
|------|------|---------|--------|
| `app-http/src/errors.rs` | 52 | `tokio::sync::Mutex` + `try_lock()` | ✅ SAFE |
| `app-http/src/todos.rs` | 25 | `tokio::sync::RwLock` + `.await` | ✅ EXEMPLARY |
| `app-http/tests/security_middleware.rs` | 47-50 | `tokio::sync::Mutex` + `.await` | ✅ SAFE |
| `xtask/src/env.rs` | 54-77 | `std::sync::Mutex` (sync-only code) | ✅ SAFE |
| `xtask/src/kernel.rs` | 230 | `std::sync::Mutex` (sync-only code) | ✅ SAFE |
| `examples/brownfield-demo/server/src/main.rs` | 68,77,91 | `std::sync::Mutex` (no await inside lock) | ✅ SAFE |

**Issue found (1/7):**

- **adapters-grpc/tests/smoke.rs:12,24,30,35,44** - Uses `std::sync::Mutex` in `#[async_trait]` methods
  - Wrong type for async context (should use `tokio::sync::Mutex`)
  - Works because locks held briefly without await inside
  - Test-only, low priority

### Impact

- **Production code:** ✅ No critical issues - all async code uses `tokio::sync::` correctly
- **Test code:** ⚠️ Smoke test uses wrong Mutex type (low impact - test-only)
- **Executor safety:** No risk of starvation - locks held for minimal time without await

### Plan

**Minimal fix:**

Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `adapters-grpc/tests/smoke.rs`:

```rust
// Change imports
- use std::sync::{Arc, Mutex};
+ use std::sync::Arc;
+ use tokio::sync::Mutex;

// Update all lock() calls in async methods
- let mut tasks = self.tasks.lock().unwrap();
+ let mut tasks = self.tasks.lock().await;
```

**Follow-ups:**
- Add doc comment to brownfield example explaining why `std::sync::Mutex` is safe there
- Consider clippy lint: `-W clippy::await_holding_lock`

**Test plan:**
```bash
cargo test -p adapters-grpc --test smoke
cargo xtask check
cargo xtask selftest
```

### Decision / Next Action

**Verdict:** Low-to-moderate priority. Production code is safe. Fix test code for correctness (not safety).

**Good news:** The codebase demonstrates correct async patterns in production:
- `app-http/src/todos.rs` is exemplary RwLock usage
- `app-http/src/errors.rs` shows proper `try_lock()` pattern
