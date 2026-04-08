## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-23 - CORS Cache Poisoning Vulnerability
**Vulnerability:** The CORS middleware dynamically reflects the `Origin` header in the `Access-Control-Allow-Origin` response header, but failed to append the `Vary: Origin` header. This could allow intermediate caches to serve a cached response intended for one origin to a different origin.
**Learning:** Whenever an application dynamically generates a response based on an incoming request header (like `Origin`), it must explicitly instruct intermediate caches to consider that header as part of the cache key by using the `Vary` header.
**Prevention:** Always append `Vary: Origin` when reflecting `Access-Control-Allow-Origin` in CORS implementations.
