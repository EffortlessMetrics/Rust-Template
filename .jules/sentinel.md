## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Missing Vary Header in CORS Responses
**Vulnerability:** When dynamically reflecting the request `Origin` header into the `Access-Control-Allow-Origin` CORS response header, the server failed to also append a `Vary: Origin` header.
**Learning:** If a response contains dynamic CORS headers based on the request's `Origin`, missing the `Vary: Origin` header could allow intermediate HTTP caches (e.g. CDNs, proxies) to cache the response tailored for one origin and incorrectly serve it to requesters from a completely different origin. This could result in unauthorized access or cache poisoning vulnerabilities.
**Prevention:** Always append `Vary: Origin` to the HTTP response headers when dynamically setting `Access-Control-Allow-Origin` based on the request header.
