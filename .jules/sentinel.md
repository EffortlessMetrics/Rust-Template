## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-02-12 - Vary: Origin caching vulnerabilities in CORS
**Vulnerability:** Intermediate cache poisoning vulnerability.
**Learning:** When the `Origin` header is dynamically reflected in the `Access-Control-Allow-Origin` CORS header (e.g., in Axum middleware), intermediate proxies or CDNs might cache the response for the wrong origin. The `Vary: Origin` header is required to prevent intermediate cache poisoning vulnerabilities, which instruct caches to key the response to the `Origin` header.
**Prevention:** When dynamically reflecting the `Origin` header in `Access-Control-Allow-Origin` CORS responses (e.g., `response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header_value);`), always append the `Vary: Origin` header (e.g., via `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));`).
