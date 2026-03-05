## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## $(date +%Y-%m-%d) - Prevent CORS Cache Poisoning
**Vulnerability:** The custom CORS middleware implementations in `app-http` and `http-middleware` omitted the `Vary: Origin` header when dynamically reflecting the `Origin` back in the `Access-Control-Allow-Origin` header. They also omitted `Vary: Access-Control-Request-Method` and `Vary: Access-Control-Request-Headers` on preflight responses.
**Learning:** Without the `Vary: Origin` header, an intermediate caching layer (like a CDN or proxy) might cache a CORS response intended for one origin and incorrectly serve it to another. This could lead to cache poisoning or broken functionality for subsequent requesters.
**Prevention:** Whenever generating a dynamic response based on a request header (such as `Origin`), ensure the corresponding `Vary` header is explicitly added. Use `append` rather than `insert` to preserve existing `Vary` directives.
