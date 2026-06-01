## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-03-01 - Middleware configuration allocations on hot path
**Learning:** Found that `SecurityHeadersConfig` in the `app-http` middleware was dynamically parsing `String` configurations into `http::HeaderValue`s using `HeaderValue::from_str` on every incoming request. This caused unnecessary per-request overhead and memory allocations for static configuration.
**Action:** Introduced a `CachedSecurityHeaders` struct to pre-parse and store configurations once at startup, which can then be cheaply cloned to provide zero-allocation header insertions during request processing. Always parse and cache middleware configuration outside the `async move` closure in Axum.
