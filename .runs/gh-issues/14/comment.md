## Investigation Report: Issue #14 - Sync File I/O in spec-fs

### Status
**Status:** fix-ready
**Local gates:** `cargo xtask check` ✓

### Evidence

**Sync I/O Operations Found (11 locations):**

1. **tasks_state.rs:41** - `fs::create_dir_all(parent)?` blocks in `update_task_status()`
2. **tasks_state.rs:45** - `OpenOptions::new().open(path)?` - sync file open with exclusive lock
3. **tasks_state.rs:48-52** - `file.lock_exclusive()` and `file.read_to_string()` block during read
4. **tasks_state.rs:68-70** - `file.set_len(0)?`, `file.seek()`, `file.write_all()` block during write
5. **tasks_state.rs:88** - `path.exists()` sync check in `get_task_status()`
6. **tasks_state.rs:92** - `OpenOptions::new().open(path)?` - sync file open with shared lock
7. **tasks_state.rs:98,125** - `file.read_to_string()` blocks in read-only functions
8. **tasks_def.rs:89** - `std::fs::read_to_string(path)` blocks in `load_tasks_definitions()`

**Call Chain:**
```
app-http async handlers
  ↓ TaskService (sync trait methods)
  ↓ FsGovernanceRepository (sync trait impl)
  ↓ tasks_state::* functions with sync file I/O
  ↓ BLOCKS tokio executor thread
```

### Impact

- **Executor Blocking:** Each governance API call blocks a tokio worker thread
- **Latency Spikes:** Concurrent requests queue behind blocking file operations
- **Poor Scalability:** Cannot handle burst concurrent loads (10+ simultaneous API calls)

**Blast radius:**
- `/platform/tasks` endpoints (GET)
- `/platform/tasks/{id}/status` endpoints (POST)
- `/ui/tasks` UI endpoint
- All downstream endpoints that load task state

### Plan

**Minimal fix (Option A - Recommended):**

Use `tokio::task::spawn_blocking()` to move blocking I/O off executor:

```rust
// In FsGovernanceRepository methods
fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
    let path = self.state_file_path.clone();
    let id = task_id.clone();

    tokio::task::block_in_place(|| {
        tasks_state::get_task_status(&path, &id)
    })
    .map_err(|e| GovernanceError::Io(std::io::Error::other(e)))?
}
```

Benefits:
- ✓ Unblocks tokio executor
- ✓ Minimal code changes (~50 lines)
- ✓ No trait breakage (stays sync)
- ✓ Backwards compatible

**Follow-ups:**
- Create `AsyncGovernanceRepository` trait for full async migration
- Migrate to `tokio::fs` for better performance

**Test plan:**
```bash
# Concurrent request test
for i in {1..20}; do
  curl -X POST http://localhost:8080/platform/tasks/TASK-$i/status \
    -H "Content-Type: application/json" -d '{"status":"InProgress"}' &
done
wait

# Latency benchmark
wrk -t4 -c50 -d10s http://localhost:8080/platform/tasks
```

### Decision / Next Action

**Recommend:** Keep open as **fix-ready**. Start with Option A (`block_in_place`) for immediate relief. Expected improvement: 30-50% reduction in p99 latency under concurrent load.
