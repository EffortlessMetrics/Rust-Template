## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2025-04-19 - Cached Security Headers (HeaderValue clones)
**Learning:** `HeaderValue::from_str` in the `http` crate is surprisingly expensive when done per-request in middleware due to string validation and allocation overhead. However, `HeaderValue::clone()` is virtually free because it's backed by `bytes::Bytes` which only performs an atomic reference count increment.
**Action:** When implementing Axum middleware using `axum::middleware::from_fn`, always pre-parse expensive configuration strings into `HeaderValue`s outside the middleware closure and cache them. Then `.clone()` those cached configurations inside the `async move` handler to eliminate per-request overhead on the hot path.
