## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Caching Header Parsing in Axum Middleware
**Learning:** Parsing strings to HeaderValues on every request in Axum middleware is a performance bottleneck. The `HeaderValue::clone()` method is extremely cheap (O(1)) in Rust's `http` crate because the underlying data is backed by `Bytes` which performs a reference count increment.
**Action:** When implementing Axum middleware, perform expensive configuration parsing (e.g., converting config strings to `HeaderValue`s) outside the middleware closure and cache the result in a struct. Then `.clone()` the cached configuration into the `async move` request handler to eliminate per-request parsing allocations on the hot path.
