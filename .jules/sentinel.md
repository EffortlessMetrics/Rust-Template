## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-30 - Fix Cache Poisoning Risk in CORS Middleware Reflection
**Vulnerability:** The CORS middleware in `crates/app-http/src/middleware/cors.rs` and `crates/http-middleware/src/cors.rs` dynamically reflected the incoming `Origin` header into the `Access-Control-Allow-Origin` response header without adding a `Vary: Origin` header. This could allow intermediate caches to serve a response intended for one origin to another, leading to cache poisoning.
**Learning:** Axum responses do not automatically add the `Vary: Origin` header when `Access-Control-Allow-Origin` dynamically reflects the request `Origin`. Symmetrical fixes are required across both crates.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` when reflecting `Origin` in CORS responses.
