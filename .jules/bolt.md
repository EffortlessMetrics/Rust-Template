## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.

## 2024-05-19 - Pre-parsing Security Headers
**Learning:** In Rust's `http` crate (used by Axum), inserting headers into an `http::HeaderMap` using a static string (e.g., `response.headers_mut().insert("Header-Name", ...)`) relies on the `IntoHeaderName` trait for `&'static str`. This requires the string to be strictly lowercase (e.g., `"header-name"`). Using uppercase characters in the static string will result in a runtime panic. Furthermore, dynamically generating a cached struct containing parsed `HeaderValue`s inside an `apply_headers` method degrades performance by introducing extra allocations on the hot path; pre-parsing into a persistent struct (`CachedSecurityHeaders`) avoids this.
**Action:** Always use lowercase header names when inserting using static string literals (e.g., `"content-security-policy"`) or use built-in constants. Separate configuration state (`SecurityHeadersConfig`) from evaluated state (`CachedSecurityHeaders`) when caching `HeaderValue`s for middleware layers.
