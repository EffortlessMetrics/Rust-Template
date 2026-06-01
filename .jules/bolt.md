## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-02-15 - Redundant String Parsing in Axum Middleware Hot Path
**Learning:** Parsing strings to `HeaderValue` inside Axum `from_fn` middleware (e.g., `HeaderValue::from_str(csp)`) incurs per-request overhead for operations that only need to happen once on application startup.
**Action:** When implementing Axum middleware, parse configurations to `HeaderValue` once during initialization and cache them (e.g., `CachedSecurityHeaders`). Then `.clone()` the cached values inside the `async move` request handler, as `HeaderValue::clone()` is extremely cheap (just a reference count increment on `Bytes`).
