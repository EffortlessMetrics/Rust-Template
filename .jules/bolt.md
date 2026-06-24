## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-02-05 - Avoid Per-Request Parsing Allocations in Axum Middleware
**Learning:** Parsing header values from strings (using `HeaderValue::from_str`) on every single request in middleware creates unnecessary allocation and string conversion overhead on the hot path.
**Action:** When implementing Axum middleware using `axum::middleware::from_fn`, perform expensive configuration parsing outside the middleware closure and cache the result. Then `.clone()` the cached configuration into the `async move` request handler. In Rust's `http` crate, `HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs a reference count increment.
