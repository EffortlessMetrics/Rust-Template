## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-15 - [Reflected CORS Allow Origin Needs Vary Header]
**Vulnerability:** CORS endpoints dynamically reflect `Access-Control-Allow-Origin` without including the `Vary: Origin` header.
**Learning:** If an application reflects `Access-Control-Allow-Origin` dynamically based on the incoming `Origin` header but does not specify `Vary: Origin`, downstream HTTP caches or intermediate proxies can cache the response. A subsequent request from a different origin might receive the cached response containing the original `Access-Control-Allow-Origin`, causing cache poisoning and either denying legitimate access or erroneously permitting access to cross-origin attackers.
**Prevention:** Always append `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` whenever `Access-Control-Allow-Origin` is inserted into a response based on the request's origin.
