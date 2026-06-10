## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-04-22 - Missing Vary: Origin header on dynamic CORS responses
**Vulnerability:** The CORS middleware dynamically reflects the `Origin` header in the `Access-Control-Allow-Origin` response header but failed to include `Vary: Origin`. This allows an intermediate caching proxy to cache the response for one origin and mistakenly serve it to requests from different (and potentially disallowed) origins.
**Learning:** Caching mechanisms are unaware of custom server logic. If a response varies based on the value of a specific request header (like `Origin`), the server must explicitly signal this to caches using the `Vary` header to prevent cache poisoning or cross-origin leakage.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` when dynamically setting `Access-Control-Allow-Origin` based on the request's origin.
