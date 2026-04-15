## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-04-15 - Axum Middleware Header Parsing Bottleneck
**Learning:** Parsing raw strings into `HeaderValue`s dynamically inside an Axum middleware (like `SecurityHeadersConfig::apply_headers`) causes unnecessary allocations and CPU overhead per request on the hot path. `http::HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs an atomic reference count increment.
**Action:** Pre-parse strings into `HeaderValue`s during configuration initialization and store them in a struct (e.g., `CachedSecurityHeaders`) inside Axum's `AppState` or the middleware closure. Ensure header names are strictly lowercase when inserting them into `Response::headers_mut()` using static string literals.
