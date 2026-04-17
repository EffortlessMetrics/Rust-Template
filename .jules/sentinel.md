## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-18 - Missing Vary: Origin in CORS
**Vulnerability:** Intermediate cache poisoning vulnerability via dynamic Origin reflection.
**Learning:** The CORS middleware dynamically reflected the request's Origin header into the Access-Control-Allow-Origin response header but failed to include a `Vary: Origin` header. This could allow an intermediate cache to serve a cached response intended for one origin to a different origin.
**Prevention:** Always append `Vary: Origin` when dynamically setting `Access-Control-Allow-Origin`.
