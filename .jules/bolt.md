## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2026-05-07 - Cached Middleware Headers
**Learning:** Pre-parsing and caching `HeaderValue`s during application startup eliminates per-request header allocation and parsing in middleware.
**Action:** Always cache heavily-used HTTP headers initialized from static configurations, especially in Axum/Tower middleware.
