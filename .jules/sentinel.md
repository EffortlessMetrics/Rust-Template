## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - CORS Cache Poisoning via Missing Vary: Origin
**Vulnerability:** The `Access-Control-Allow-Origin` header dynamically reflected the incoming `Origin` header without setting `Vary: Origin`. This allows intermediate caching layers (e.g., proxies or CDNs) to cache the ACAO header intended for one origin and serve it to a different origin.
**Learning:** When dynamically generating ACAO headers based on incoming requests, failing to instruct caches to vary by the `Origin` header introduces a severe cache poisoning vector. The caching mechanism needs to know that the response differs depending on the `Origin`.
**Prevention:** Whenever CORS headers dynamically reflect an incoming `Origin` instead of a static allowed list, ensure `response.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));` is included to correctly scope the cached responses.
