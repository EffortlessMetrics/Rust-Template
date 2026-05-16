## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - CORS Cache Poisoning via Missing Vary: Origin
**Vulnerability:** The CORS middleware dynamically reflected the `Origin` header into `Access-Control-Allow-Origin` but failed to append `Vary: Origin` to the response headers. This omission allows intermediate caches (like CDNs) to cache a response tailored to one origin and mistakenly serve it to subsequent requests from different origins, leading to cache poisoning.
**Learning:** When dynamically generating response headers based on request headers (like CORS), you must explicitly tell caches that the response varies based on that input header using the `Vary` header.
**Prevention:** Always append `Vary: Origin` when reflecting `Access-Control-Allow-Origin`. Use `append` instead of `insert` for multi-value headers like `Vary` in Axum/Hyper to avoid overwriting existing directives.
