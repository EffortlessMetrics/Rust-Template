## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - CORS Cache Poisoning Risk
**Vulnerability:** The CORS implementation dynamically reflected the incoming `Origin` into the `Access-Control-Allow-Origin` header without also sending a `Vary: Origin` header. This allows an intermediate cache to incorrectly serve a response meant for one origin to a different origin, leading to a cache-poisoning bypass.
**Learning:** Whenever CORS headers are dynamic (e.g. based on a whitelist check against the request's Origin), the response MUST include `Vary: Origin` to ensure intermediate caches key their entries by the `Origin` header.
**Prevention:** Always append `Vary: Origin` when reflecting an allowed origin. Use `.append()` instead of `.insert()` to preserve existing `Vary` headers (like `Accept-Encoding`). This logic is duplicated in both `app-http` and `http-middleware` and must be applied consistently.
