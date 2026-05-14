## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-23 - Prevent Cache Poisoning via Dynamic CORS
**Vulnerability:** Dynamically reflecting the `Origin` header into `Access-Control-Allow-Origin` without a `Vary: Origin` header allows intermediate caches to incorrectly serve a cached response to a different origin.
**Learning:** Always append `Vary: Origin` whenever `Access-Control-Allow-Origin` is set dynamically based on the request.
**Prevention:** Update CORS middleware to safely append `Vary: Origin` while preserving existing `Vary` headers.
