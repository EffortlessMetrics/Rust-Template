## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Caching HeaderValues in Axum Middleware
**Learning:** Parsing strings into `HeaderValue`s using `HeaderValue::from_str` within `axum` middleware closures on every request causes unnecessary allocations and latency. `HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs a reference count increment.
**Action:** Pre-parse expensive headers into a cached configuration struct (e.g., `CachedSecurityHeaders`) when the middleware is initialized or applied, and `.clone()` them inside the `async move` request handler closure to eliminate per-request parsing allocations.
