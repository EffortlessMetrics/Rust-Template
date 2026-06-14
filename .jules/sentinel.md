## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-18 - [CRITICAL] Prevent Cache Poisoning in Dynamic CORS Responses
**Vulnerability:** When dynamically reflecting the `Origin` header into the `Access-Control-Allow-Origin` response, the response was not marked with a `Vary: Origin` header. This allows intermediate caches (like CDNs or proxies) to cache a response allowed for one origin and mistakenly serve it to subsequent requests from different, unallowed origins, leading to data exposure and cache poisoning.
**Learning:** Any dynamic CORS configuration that determines `Access-Control-Allow-Origin` based on the request's `Origin` must explicitly tell downstream caches that the response varies based on that origin.
**Prevention:** Always ensure `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` is present alongside any dynamic `ACCESS_CONTROL_ALLOW_ORIGIN` headers.
