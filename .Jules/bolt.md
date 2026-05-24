## 2024-05-23 - Blocking I/O in Async Handlers
**Learning:** The codebase contained multiple instances of synchronous file I/O operations (e.g., `fs::read_to_string`, `load_all_specs`) directly within `async` axum handlers. This blocks the executor thread, degrading concurrency.
**Action:** Always wrap synchronous blocking operations in `tokio::task::spawn_blocking` within async handlers to offload them to a dedicated thread pool. Ensure captured variables (like `PathBuf`) are cloned and moved into the `'static` closure.
