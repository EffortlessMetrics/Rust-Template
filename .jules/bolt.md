## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-01-30 - Cached Middleware Configuration
**Learning:** Found repetitive parsing of static configuration (security headers) in request hot paths. `HeaderValue::from_str` involves validation and allocation on every request.
**Action:** Pre-compute and cache derived values (like `HeaderValue`) in `AppState` or middleware structs during initialization to avoid per-request parsing overhead.
