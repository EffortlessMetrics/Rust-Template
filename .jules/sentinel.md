## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-23 - Intermediate Cache Poisoning Risk in CORS Origin Reflection
**Vulnerability:** Reflecting the `Origin` header in `Access-Control-Allow-Origin` without a `Vary: Origin` header can lead to intermediate cache poisoning.
**Learning:** If an intermediate cache stores a response containing `Access-Control-Allow-Origin: <malicious_origin>`, subsequent requests from valid origins might receive the cached response, causing CORS failures or potential cross-origin access leaks.
**Prevention:** Always append `Vary: Origin` when dynamically setting `Access-Control-Allow-Origin` based on the incoming request's `Origin` header.
