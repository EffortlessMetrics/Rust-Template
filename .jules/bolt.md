## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2025-01-01 - Middleware Request Parsing Overhead
**Learning:** Axum's `HeaderValue` implements a cheap `clone()` because the underlying `Bytes` type is reference-counted. However, parsing strings into `HeaderValue`s using `HeaderValue::from_str` involves per-request string parsing and potential allocation which is unnecessary overhead.
**Action:** Always pre-parse configuration strings into `HeaderValue` outside the middleware closure (e.g., storing them in a struct like `CachedSecurityHeaders`), and simply `.clone()` them per-request to eliminate hot-path allocations.
