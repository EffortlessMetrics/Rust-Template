## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Intermediate Cache Poisoning in CORS
**Vulnerability:** CORS middleware reflected the request's `Origin` header in the `Access-Control-Allow-Origin` response without appending a `Vary: Origin` header. This allowed intermediate caches to store the response and serve the allowed origin's headers to different origins.
**Learning:** Whenever CORS headers dynamically reflect an origin, a `Vary: Origin` header is required to instruct caches to separate entries based on the `Origin` header.
**Prevention:** Always append `Vary: Origin` header alongside dynamically generated `Access-Control-Allow-Origin` values in all HTTP middleware implementations.
