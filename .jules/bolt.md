## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-28 - Axum Middleware Header Parsing Bottleneck
**Learning:** Middleware (e.g., Security Headers, CORS) that parses configurations into Axum `HeaderValue`s using `HeaderValue::from_str` per-request creates unnecessary CPU overhead. In the `http` crate, `HeaderValue::clone()` is backed by `Bytes`, meaning it performs an extremely cheap reference count increment rather than string parsing or memory allocation.
**Action:** When implementing Axum middleware, parse string configurations into `HeaderValue`s during application initialization/building and cache them. Use `cached.clone()` in the hot path of the middleware closure.
