## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-04-16 - Prevent CORS Cache Poisoning
**Vulnerability:** Dynamically reflected Origin headers in CORS without Vary: Origin protection
**Learning:** When generating Access-Control-Allow-Origin dynamically based on the request's Origin, omitting Vary: Origin allows intermediate caches to serve a response tailored for one origin to a different origin, leading to cache poisoning.
**Prevention:** Always append Vary: Origin when dynamically reflecting the Origin header in CORS responses.
