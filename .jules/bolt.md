## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-01-27 - Isolating Initialization in Regression Tests
**Learning:** Regression tests for blocking I/O must isolate the handler execution from the application initialization (which is often inherently blocking). Failing to do so can lead to false negatives or noisy tests where initialization overhead masks the actual handler performance.
**Action:** When testing handler performance, initialize the `App` or `Router` outside the concurrent request loop.
