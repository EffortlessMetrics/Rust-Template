## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-01-20 - Fix Intermediate Cache Poisoning in CORS
**Vulnerability:** CORS wildcard logic and dynamic reflected origins missing `Vary: Origin` header could lead to intermediate cache poisoning.
**Learning:** When configuring CORS with a reflected Origin (dynamically allowed), the `Vary: Origin` header must be explicitly set (using `append`) in both preflight and regular responses to prevent intermediate cache poisoning.
**Prevention:** Always append `Vary: Origin` whenever an `Access-Control-Allow-Origin` header is added dynamically based on the incoming request's `Origin` header.
