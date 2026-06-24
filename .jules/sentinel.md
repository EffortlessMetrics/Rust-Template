## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2026-03-20 - CORS Reflected Origin Missing Vary Header
**Vulnerability:** CORS middleware dynamically set Access-Control-Allow-Origin based on incoming Origin header but failed to include Vary: Origin.
**Learning:** This exposes the application to cache poisoning where intermediate proxies cache responses for an unauthorized origin.
**Prevention:** Always append Vary: Origin header when Access-Control-Allow-Origin is dynamically set based on the request's Origin.
