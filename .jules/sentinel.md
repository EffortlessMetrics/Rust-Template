## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-02-24 - Missing Vary Header in CORS Responses Allows Cache Poisoning
**Vulnerability:** The CORS middleware implementation reflected the incoming `Origin` header dynamically into the `Access-Control-Allow-Origin` response header, but failed to include a `Vary: Origin` header alongside it.
**Learning:** If a dynamic `Origin` is reflected without `Vary: Origin`, intermediate caches (like CDNs or reverse proxies) might cache the response for the first requester's origin and incorrectly serve it to subsequent requesters with different origins, potentially bypassing CORS controls or poisoning the cache.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` whenever dynamically returning an `Access-Control-Allow-Origin` header based on the incoming `Origin`.
