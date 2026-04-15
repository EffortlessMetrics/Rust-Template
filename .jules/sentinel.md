## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-04-15 - Prevent Intermediate Cache Poisoning in CORS
**Vulnerability:** Setting 'Access-Control-Allow-Origin' dynamically based on the request's 'Origin' without a 'Vary: Origin' header can lead to intermediate cache poisoning, where a cached response allowing a malicious origin might be served to other users.
**Learning:** The Axum CORS middleware in both 'app-http' and 'http-middleware' was dynamically setting the origin but failing to append the Vary header.
**Prevention:** Whenever 'Access-Control-Allow-Origin' is set dynamically in response to a request header, always append 'Vary: Origin' to the response headers.
