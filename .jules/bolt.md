## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-18 - Pre-parsing Axum Middlewares
**Learning:** Axum middleware that parses configuration strings into `HeaderValue` per-request causes unnecessary string parsing allocations on the hot path. `HeaderValue::from_str` can be expensive.
**Action:** Always pre-parse configuration into `HeaderValue`s outside the middleware closure and cache them. Since `HeaderValue::clone()` is backed by `Bytes` (reference counted), cloning the pre-parsed headers in the `async move` handler is extremely cheap and eliminates per-request allocations.
