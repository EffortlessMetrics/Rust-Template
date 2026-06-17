## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-19 - [Eliminating allocations on the HTTP middleware hot path]
**Learning:** Axum's `HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs a reference count increment. `HeaderName::from_static` avoids allocations entirely.
**Action:** When implementing Axum middleware via `axum::middleware::from_fn`, perform expensive configuration parsing (e.g., converting config strings to `HeaderValue`s) outside the middleware closure and cache the result. Then `.clone()` the cached pre-parsed configurations into the request handler to eliminate per-request parsing allocations on the hot path.
