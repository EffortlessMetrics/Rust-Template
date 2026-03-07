## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-30 - Prevent CORS Intermediate Cache Poisoning
**Vulnerability:** The CORS middleware set dynamically calculated allowed origins based on the request's `Origin` header, but failed to append the `Vary: Origin` header to the response.
**Learning:** This lack of a `Vary` header causes intermediate caches (like CDNs) to cache the first allowed CORS response. Subsequent requests from different allowed origins could be served the cached response intended for the first origin, breaking CORS and potentially leaking data or allowing unauthorized cross-origin access.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("Origin"));` whenever a response includes dynamically generated CORS headers based on the request's origin. Ensure this is done for both successful preflight responses, forbidden preflight responses, and standard responses.
