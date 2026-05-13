## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-05-13 - CORS Cache Poisoning Prevention
**Vulnerability:** Dynamically reflecting the `Origin` header in the `Access-Control-Allow-Origin` response header without appending `Vary: Origin` can lead to intermediate cache poisoning, where a malicious origin is cached and served to legitimate users.
**Learning:** Middlewares handling CORS must always set `Vary: Origin` to ensure intermediate caches store separate responses for different `Origin` requests, preventing cache poisoning vulnerabilities.
**Prevention:** In Axum or any web framework, whenever dynamically setting the `Access-Control-Allow-Origin` header based on the request's origin, always append the `Vary: Origin` header using `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));`.
