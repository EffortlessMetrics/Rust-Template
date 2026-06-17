## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Pre-parsing HTTP Headers for Zero-Allocation Middleware
**Learning:** Found that `SecurityHeadersConfig` was parsing string slices into `HeaderValue`s using `HeaderValue::from_str(...)` on every single HTTP response in the middleware hot path. This triggered frequent allocations and validation overhead. In Rust's `http` crate, `HeaderValue::clone()` is highly optimized (often just an atomic reference count increment via `Bytes`).
**Action:** When writing Axum middleware that applies static HTTP headers from a configuration, pre-parse the strings into `HeaderValue`s at application startup, store them in a cached struct in `AppState`, and use `.clone()` inside the `from_fn` closure to eliminate per-request parsing overhead.
