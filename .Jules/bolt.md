## 2024-05-23 - Sync File I/O in Async UI Handlers
**Learning:** The `http-platform` UI handlers (`dashboard`, etc.) perform synchronous file I/O via `spec_runtime::load_*` functions directly in the async handler, blocking the reactor.
**Action:** Wrap all `spec_runtime` loading calls in `tokio::task::spawn_blocking` within UI handlers to ensure non-blocking execution.
