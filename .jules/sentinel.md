## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - Missing Vary Header During Dynamic CORS Reflection
**Vulnerability:** The dynamic `Origin` header reflection in CORS middlewares (e.g., `crates/http-middleware/src/cors.rs`, `crates/app-http/src/middleware/cors.rs`) was missing the accompanying `Vary: Origin` header.
**Learning:** If `Vary: Origin` is omitted while dynamically reflecting an `Origin` inside `Access-Control-Allow-Origin`, intermediate caches (like CDNs or proxies) might cache the response for one origin and serve it to subsequent requests from different origins, leading to cache poisoning and unauthorized cross-origin access.
**Prevention:** Always append `Vary: Origin` when dynamically reflecting the `Origin` header in `Access-Control-Allow-Origin` responses.
