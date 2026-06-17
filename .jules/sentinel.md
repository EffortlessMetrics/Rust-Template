## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2026-04-24 - Intermediate Cache Poisoning in CORS
**Vulnerability:** CORS middleware dynamically reflecting the `Origin` header into the `Access-Control-Allow-Origin` response without appending a `Vary: Origin` header introduces an intermediate cache poisoning vulnerability where an aggressive intermediary cache might serve an allowed response to an unauthenticated origin.
**Learning:** Always pair dynamic `Access-Control-Allow-Origin` reflections with a `Vary: Origin` header to explicitly signal to caches that the response is strictly scoped to the requesting origin.
**Prevention:** Incorporate a linting rule or explicit review guideline for CORS implementations to guarantee `Vary` accompanies dynamic `Origin` handling.
