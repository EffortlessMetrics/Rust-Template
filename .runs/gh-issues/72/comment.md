## Investigation Report: Issue #72 - Async Runtime Safety

### Status
**Status:** triaged / fix-ready
**Local gates:** grep searches for fs::, std::fs, Mutex patterns; code reading of async handlers

### Evidence

**BLOCKING I/O IN ASYNC HANDLERS (Critical):**

1. **`crates/app-http/src/platform.rs:437`** - blocking read in `async fn get_status`
   - Issue: `fs::read_to_string(policy_path)` reads policy status file synchronously
   - Impact: Blocks tokio executor thread; under concurrent requests, delays all other tasks

2. **`crates/app-http/src/platform.rs:526`** - blocking directory read in helper
   - Function: `fn load_question_counts()` called from `async fn get_status`
   - Issue: `fs::read_dir()` with multiple `fs::read_to_string()` calls
   - Impact: Iterates all question files synchronously

3. **`crates/app-http/src/platform.rs:596`** - blocking directory iteration
   - Function: `fn load_friction_counts()` called from `async fn get_status`
   - Issue: `fs::read_dir()` with multiple `fs::read_to_string()` calls

4. **`crates/app-http/src/platform.rs:667`** - blocking read in helper
   - Function: `fn load_fork_counts()` called from `async fn get_status`
   - Issue: `fs::read_to_string(&registry_path)` reads fork registry synchronously

5. **`crates/gov-http/src/handlers.rs:63`** - blocking read in `async fn get_openapi`
   - Issue: `fs::read_to_string(&openapi_path)` reads OpenAPI YAML synchronously
   - Impact: Every platform schema request holds up the runtime

6. **`crates/gov-http/src/handlers.rs:420`** - blocking read in coverage calculation
   - Function: `async fn get_coverage`
   - Issue: `std::fs::read_to_string(&bdd_json_path)` reads BDD report synchronously

**ANTI-PATTERN IN EXAMPLES (High Priority):**

7. **`examples/brownfield-demo/server/src/main.rs:12,27,68,77,91`**
   - Type: `type SharedState = Arc<Mutex<Vec<Item>>>;` (std::sync::Mutex)
   - Handlers: `list_items`, `create_item`, `get_item`
   - Issue: `state.lock().expect(...)` blocks tokio executor during lock acquisition
   - Impact: Example teaches anti-pattern; developers copying this cause executor starvation

### Impact

- **Performance**: Status endpoint blocks executor thread for 10-100ms per concurrent request
- **Correctness**: Architecture violates tokio guarantees
- **Developer Experience**: Example code teaches wrong pattern (std::sync::Mutex with axum)
- **Blast Radius**:
  - Direct: `/platform/status`, `/platform/openapi`, `/platform/coverage`, task status updates
  - Indirect: All endpoints in same axum service affected by executor starvation
  - Example: Every brownfield-demo derivative copies the mutex pattern

### Plan

**Minimal fix:**

1. **Replace blocking file I/O with tokio::fs** in async contexts:
   - `crates/app-http/src/platform.rs`:
     - Line 437: Replace `fs::read_to_string()` with `tokio::fs::read_to_string().await`
     - Lines 526, 536: Replace with `tokio::fs::read_dir()` and async reads
     - Lines 596, 606: Same for friction
     - Line 667: Replace with `tokio::fs::read_to_string().await`

   - `crates/gov-http/src/handlers.rs`:
     - Line 63: Replace with `tokio::fs::read_to_string().await`
     - Line 420: Replace with `tokio::fs::read_to_string().await`

2. **Make helper functions async**:
   - `load_question_counts()` → `async fn`
   - `load_friction_counts()` → `async fn`
   - `load_fork_counts()` → `async fn`

3. **Fix brownfield-demo example**:
   - Replace `Arc<Mutex<Vec<Item>>>` with `Arc<tokio::sync::RwLock<Vec<Item>>>`
   - Update handlers to use `.read().await` and `.write().await`
   - Update comment to explain why tokio::sync is required

**Follow-ups:**
- Evaluate whether `FsGovernanceRepository` needs to be async
- Consider caching spec files instead of re-reading on every status request
- Add clippy lint: `cargo clippy -- -W clippy::await_holding_lock` to CI
- Benchmark `/platform/status` before/after

**Test plan:**
```bash
cargo test -p app-http
cargo xtask ac-tests AC-*
# Stress test (optional)
wrk -t4 -c50 -d10s http://localhost:8080/platform/status
```

### Decision / Next Action

**Recommend:** Keep issue open, mark as **fix-ready**. This is a known anti-pattern with clear fix.

**Priority order:**
1. Fix crates/app-http/src/platform.rs (affects main service)
2. Fix crates/gov-http/src/handlers.rs (affects platform APIs)
3. Fix examples/brownfield-demo (prevents anti-pattern spread)
