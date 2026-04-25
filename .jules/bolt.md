## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Pre-parsing Axum Middleware Headers
**Learning:** Found multiple instances of security header string parsing inside `Axum` middleware on every request (`HeaderValue::from_str`).
**Action:** Always pre-parse configuration strings into `HeaderValue`s outside the middleware and cache them. `HeaderValue::clone()` is extremely cheap because it is backed by `Bytes` (reference count increment), eliminating per-request parsing overhead.
