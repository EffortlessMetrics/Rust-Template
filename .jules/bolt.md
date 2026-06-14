## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-02-05 - Avoid blocking the event loop with synchronous file I/O
**Learning:** Avoid blocking the Tokio event loop with heavy synchronous file I/O. Use `tokio::task::spawn_blocking` when reading multiple files or parsing large YAML files in an asynchronous Axum handler.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
