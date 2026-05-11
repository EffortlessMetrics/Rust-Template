## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-11 - [Zero-Allocation Security Headers in Axum Middleware]
**Learning:** In Axum middleware using `axum::middleware::from_fn`, converting configuration values to `HeaderValue`s per-request causes repeated parsing allocations. In the `http` crate, `HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs a reference count increment.
**Action:** When implementing middleware logic in `app-http` and `http-middleware` that relies on configuration structures like `SecurityHeadersConfig`, perform expensive string parsing once, store it in a cached structure (e.g. `CachedSecurityHeaders`), and then `.clone()` the cached configuration into the `async move` request handler. This eliminates per-request allocations.
