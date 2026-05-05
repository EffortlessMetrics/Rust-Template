## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-17 - [Axum Middleware Configuration Caching]
**Learning:** In Axum, `axum::middleware::from_fn` evaluates on the hot path per request. Parsing configuration strings into `HeaderValue`s per request creates unnecessary overhead. In Rust's `http` crate, `HeaderValue::clone()` is extremely cheap (backed by reference-counted `Bytes`), so pre-parsing configs into `HeaderValue`s when the middleware starts up and cloning them inside the `async move` closure is significantly more performant.
**Action:** Always pre-parse string-based configurations into `HeaderValue` or other typed Axum/http structs outside the middleware closure, and `.clone()` them in.
