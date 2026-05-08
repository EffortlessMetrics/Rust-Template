## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-05-08 - Blocking I/O in Async Handlers
**Learning:** Found synchronous spec loading (`generate_snapshot`) blocking the async executor in `get_idp_snapshot`. Wrapping it in `tokio::task::spawn_blocking` and handling the `JoinError` fixes the starvation issue.
**Action:** Consistently offload synchronous I/O and heavy parsing (e.g., `serde_yaml` from files) to blocking threads in async endpoints.
