## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-06-15 - CORS Dynamic Origin Cache Poisoning
**Vulnerability:** The CORS middleware reflected the `Origin` header dynamically in `Access-Control-Allow-Origin` without setting `Vary: Origin`.
**Learning:** Intermediate caches could serve a response cached for one origin to a different origin, leading to cache poisoning.
**Prevention:** Always append `Vary: Origin` when dynamically reflecting `Origin` in CORS headers. Use `append` instead of `insert` for multi-value headers like `Vary`.
