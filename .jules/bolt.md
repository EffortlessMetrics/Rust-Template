## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2026-01-27 - Zero-Allocation Security Headers
**Learning:** Axum's `HeaderValue::from_str` and `TryFrom<&str>` for headers (like complex Content-Security-Policy strings) incur validation and allocation overhead. Dynamically generating a cache structure with these on every request (via dynamic methods) defeated the purpose of caching. `HeaderValue` is extremely cheap to clone (atomic ref-count for long strings, inline copy for short).
**Action:** When optimizing middleware configurations, pre-parse string headers into a `Cached` struct at application startup, store that cached struct in Axum's `AppState` or clone it outside the `from_fn` layer, and use `HeaderValue::clone()` on the hot path for zero-allocation performance. Ensure legacy dynamic methods do not internally generate the cache struct on the hot path.
