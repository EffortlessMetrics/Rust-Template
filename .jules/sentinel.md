## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-27 - Intermediate cache poisoning via CORS dynamic origin reflection
**Vulnerability:** The CORS middleware dynamically reflected the incoming `Origin` header in the `Access-Control-Allow-Origin` response header without setting the `Vary: Origin` header.
**Learning:** If a CDN or intermediate proxy caches a response that dynamically reflects the `Origin` header without `Vary: Origin`, subsequent requests from different origins might receive the cached response intended for the first origin, potentially leading to unauthorized access or broken CORS for legitimate clients.
**Prevention:** When dynamically setting `Access-Control-Allow-Origin` based on the request's origin, always append `Vary: Origin` to the response headers to instruct caches to vary their responses based on the request's origin.
