## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-02-26 - Pre-parsing HeaderValue in Axum middleware
**Learning:** Axum's `HeaderValue` parsing from strings allocates and takes time on every request. By pre-parsing these values into a cached struct and calling `.clone()` inside the `async move` middleware closure, we can significantly reduce per-request overhead, since `HeaderValue::clone()` is a cheap reference count increment backed by `Bytes`.
**Action:** Always pre-parse configuration strings into `HeaderValue`s outside the request hot path in middleware.
