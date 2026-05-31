## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-23 - CORS Cache Poisoning via Reflected Origin
**Vulnerability:** The CORS middleware reflected the `Origin` header dynamically into the `Access-Control-Allow-Origin` response without setting the `Vary: Origin` header. This allowed an attacker's cross-origin response to be cached by intermediate proxies, poisoning the cache for legitimate users on different origins.
**Learning:** Whenever generating a dynamic response that depends on a request header (such as `Origin`), you must indicate this to intermediate caches. Furthermore, when setting the `Vary` header in Axum/Hyper, you must use `.append()` instead of `.insert()` to avoid overwriting existing `Vary` directives (like `Vary: Accept-Encoding`).
**Prevention:** Always set `Vary: Origin` (using `append`) when dynamically reflecting the `Access-Control-Allow-Origin` based on the request.
