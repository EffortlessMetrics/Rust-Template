## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2024-03-24 - Zero-overhead Middleware Headers
**Learning:** Parsing HTTP header string configurations into `HeaderValue`s on every request creates unnecessary parsing overhead in Axum/Tokio middleware.
**Action:** Always pre-parse static header configurations (like SecurityHeadersConfig) into cached `HeaderValue`s at startup and clone them per request. The `HeaderValue` type is cheap to clone.
