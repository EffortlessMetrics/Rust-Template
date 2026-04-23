## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-02-14 - Intermediate Cache Poisoning in Dynamic CORS
**Vulnerability:** CORS middleware dynamically reflected the incoming `Origin` header in the `Access-Control-Allow-Origin` response without appending a `Vary: Origin` header. This allows intermediate caches (like CDNs or proxies) to cache the response for one origin and mistakenly serve it to subsequent requests from different, potentially unauthorized origins.
**Learning:** When dynamically setting CORS headers based on the request's `Origin`, it is critical to instruct caches to vary their responses based on the `Origin` header to prevent cross-origin data leakage or cache poisoning.
**Prevention:** Always append `Vary: Origin` (e.g., `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));`) whenever dynamically returning `Access-Control-Allow-Origin`.
