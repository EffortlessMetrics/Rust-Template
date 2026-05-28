## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-02-27 - CORS Vary Origin Header
**Vulnerability:** When dynamically reflecting the `Origin` header in the `Access-Control-Allow-Origin` CORS response header without adding `Vary: Origin`, intermediate caches (like CDNs or proxies) can incorrectly cache the response for one origin and serve it to clients from a different origin. This leads to broken CORS for legitimate users or potential cache poisoning attacks where an attacker can force a cached response with their own allowed origin.
**Learning:** Any dynamic modification of response headers based on request headers (like `Origin`) must be accompanied by the appropriate `Vary` header to instruct caches to separate entries based on the dynamic request header.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` whenever setting `Access-Control-Allow-Origin` to a dynamically extracted origin value.
