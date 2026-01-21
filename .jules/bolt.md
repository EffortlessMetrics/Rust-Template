## 2024-05-23 - Blocking I/O in Async Handlers
**Learning:** `app-http` handlers perform synchronous file I/O (via `spec_runtime`) directly in async functions. This blocks the Tokio executor thread, potentially starving other tasks (like timers or concurrent requests).
**Action:** Always wrap synchronous spec loading (e.g., `load_all_specs`, `load_tasks`) in `tokio::task::spawn_blocking`.
