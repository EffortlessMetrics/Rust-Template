## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-06 - Pre-parsing Axum Middleware Headers
**Learning:** Found that `axum::http::HeaderValue::from_str` was being called inside the middleware closure for every single request, causing unnecessary string parsing and allocations. Since the configuration strings don't change after startup, they should be pre-parsed.
**Action:** Always parse static header strings into `axum::http::HeaderValue` outside the `axum::middleware::from_fn` closure and `.clone()` them inside. `HeaderValue::clone()` is extremely cheap (just a reference count increment) since the underlying string is backed by `Bytes`.
