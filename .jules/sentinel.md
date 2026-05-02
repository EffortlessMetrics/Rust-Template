## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-25 - Intermediate Cache Poisoning in CORS
**Vulnerability:** A CORS implementation explicitly reflected the `Origin` header to `Access-Control-Allow-Origin` but did not append the `Vary: Origin` header. This could cause intermediate caches (like CDNs or proxies) to cache the first observed origin as the allowed origin for all subsequent requests, resulting in potential cache poisoning or access disruption.
**Learning:** When dynamically setting CORS headers based on the request`s `Origin` header, the response MUST include `Vary: Origin` so intermediate HTTP caches use the origin when keying cached responses.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` whenever `Access-Control-Allow-Origin` reflects the request origin.
