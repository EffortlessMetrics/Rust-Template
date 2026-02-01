## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-01-28 - Cloning Data for Spawn Blocking
**Learning:** When moving data into a `tokio::task::spawn_blocking` closure, references to `AppState` (like `&Path`) cannot be used because the closure must be `'static`.
**Action:** Clone the necessary data (e.g., `state.workspace_root().to_path_buf()`) before spawning the task and move the owned data into the closure.
