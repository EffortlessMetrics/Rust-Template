## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2026-04-19 - CORS Cache Poisoning
**Vulnerability:** Dynamic CORS responses reflecting the user's Origin in Access-Control-Allow-Origin were missing the Vary: Origin header.
**Learning:** When dynamically reflecting the Origin header in Access-Control-Allow-Origin CORS responses, intermediate proxies or CDNs can cache the response for one origin and mistakenly serve it to a different origin, leading to broken cross-origin requests or potential cache poisoning vulnerabilities.
**Prevention:** Always append the Vary: Origin header when returning dynamic Access-Control-Allow-Origin headers to explicitly instruct intermediate caches to key the response based on the Origin header.
