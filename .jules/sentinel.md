## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Missing Vary: Origin header in dynamic CORS configuration
**Vulnerability:** The CORS middleware dynamically reflected the incoming `Origin` into `Access-Control-Allow-Origin` without setting `Vary: Origin`.
**Learning:** This could lead to cache poisoning if a caching proxy caches a response intended for one allowed origin and serves it to a completely different origin. Since the allowed origin is reflected dynamically, caches need to know that the response varies based on the request's origin.
**Prevention:** Always append the `Vary: Origin` header when `Access-Control-Allow-Origin` is populated dynamically.
