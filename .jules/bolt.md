## 2026-01-27 - Blocking I/O in Async Handlers
**Learning:** Found multiple instances of synchronous file I/O (std::fs, spec loading) inside async Axum handlers. This blocks the Tokio worker thread.
**Action:** Always wrap heavy synchronous operations (like loading YAML specs) in `tokio::task::spawn_blocking` to keep the runtime responsive.
## 2024-05-14 - Pre-parsing header values in middleware
**Learning:** Axum middleware that applies HTTP headers often parses strings into `HeaderValue`s using `HeaderValue::from_str` on every single request, which adds unnecessary parsing overhead on the hot path. In `security_headers_middleware`, up to 10 strings are parsed to header values per request.
**Action:** Use a pre-parsed struct of `HeaderValue`s that is generated once during configuration and then cheap to clone into responses, using `Option<HeaderValue>` for optional headers.
