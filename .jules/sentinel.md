## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Intermediate Cache Poisoning in CORS Reflection
**Vulnerability:** The CORS middleware was reflecting the requested `Origin` in the `Access-Control-Allow-Origin` header without appending the `Vary: Origin` header.
**Learning:** If an intermediate cache (like a CDN or reverse proxy) caches a response with a reflected origin but without `Vary: Origin`, it might serve that cached response (with the original requester's origin in the ACAO header) to a subsequent requester from a completely different origin. This breaks CORS security for the subsequent requester and can lead to cache poisoning vulnerabilities where malicious origins can bypass CORS checks.
**Prevention:** When dynamically reflecting the `Origin` header in `Access-Control-Allow-Origin` responses, always append the `Vary: Origin` header to instruct caches to separate cache entries based on the `Origin` header of the request.
