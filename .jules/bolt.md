## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-24 - Pre-parsed Cached Configurations in Axum Middleware
**Learning:** Middleware configurations in Axum applications (e.g. `SecurityHeadersConfig` in `crates/app-http/src/lib.rs`) that parse headers on every request on the hot path should be optimized by caching their parsed `HeaderValue`s using a struct like `CachedSecurityHeaders`. `HeaderValue::clone()` is extremely cheap (backed by `Bytes` reference counting) and avoids repeated string allocations and parsing operations per request.
**Action:** Implemented `CachedSecurityHeaders` which maps strings to `Option<HeaderValue>`, introduced a `.cache()` method on `SecurityHeadersConfig`, and updated Axum middleware layers to `.clone()` the cached values into handlers.
