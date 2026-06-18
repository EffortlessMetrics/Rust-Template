## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-01-27 - Security Headers Middleware Optimization
**Learning:** Security headers middleware was parsing strings into `HeaderValue`s per request. This creates unnecessary allocations and string parsing overhead on the hot path for static configuration values.
**Action:** Introduced `CachedSecurityHeaders` struct that parses string config into `HeaderValue` once on application startup or config load. Cached values are then cheaply cloned (atomic ref count increment by `Bytes`) into responses per request.
