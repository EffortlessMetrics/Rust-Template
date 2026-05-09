## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-18 - Cached Security Headers Middleware

**Learning:** When implementing Axum middleware using `axum::middleware::from_fn` or `axum::middleware::from_fn_with_state` and performing configuration parsing that involves converting configurations to `HeaderValue`s, you can pre-parse and cache `HeaderValue`s to eliminate per-request allocations. `HeaderValue::clone()` is extremely cheap because the underlying data is backed by `Bytes` which performs a reference count increment. This makes pre-parsing and cloning header values in middleware significantly more performant than per-request string parsing.

**Action:** Identify middleware parsing and construct a `CachedConfig` structure at server initialization to pre-parse configs into `HeaderValue`s, then pass the cached structure inside Axum closures and `clone` into the request scope to eliminate per-request string-parsing hot-paths.
