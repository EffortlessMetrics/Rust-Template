## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2024-05-19 - Middleware Configuration Caching Pattern
**Learning:** Axum/Hyper `HeaderValue::from_str` allocates and parses strings. Calling it on every request for static configurations like `SecurityHeadersConfig` introduces unnecessary CPU overhead and allocation.
**Action:** When working with static middleware configurations (e.g., security headers, CORS), introduce a cached variant (like `CachedSecurityHeaders`) that eagerly evaluates strings into `HeaderValue` during application startup (`AppState` creation). Serve these pre-parsed values via cloning (which utilizes cheap atomic ref-counts for `HeaderValue`) in the hot path. Remember to leave original dynamic/inline parsing functions intact.
