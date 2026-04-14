## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Intermediate Cache Poisoning in Dynamic CORS
**Vulnerability:** The application dynamically reflected the request's `Origin` header into the `Access-Control-Allow-Origin` response header, but failed to include a `Vary: Origin` header. This could allow an intermediate cache (like a CDN) to cache a response meant for one origin and serve it to a different origin, leading to a cache poisoning or data leakage risk.
**Learning:** When generating HTTP responses that vary based on a specific request header (like `Origin`), the server must explicitly signal this to downstream caches by appending the `Vary` header. This is a common and subtle pitfall when implementing custom CORS logic.
**Prevention:** Always append `Vary: Origin` when setting `Access-Control-Allow-Origin` dynamically, or rely on robust frameworks/libraries that handle CORS compliance automatically.
