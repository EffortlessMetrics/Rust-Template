## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2026-06-05 - Fix dynamic Origin reflection missing Vary header
**Vulnerability:** When a server dynamically reflects the request's `Origin` header into the response's `Access-Control-Allow-Origin` header, caching intermediaries (like CDNs, proxies, or browsers) might cache the response and inappropriately serve it to a different origin. This can either break legitimate cross-origin requests or inadvertently allow unauthorized cross-origin access.
**Learning:** Adding `Vary: origin` instructs caches to key their cached responses based on the request's `Origin` header, successfully mitigating this risk. In Rust/Axum, `header_mut().append(header::VARY, HeaderValue::from_static("origin"))` should be used to not overwrite existing `Vary` headers.
**Prevention:** Always append `Vary: Origin` when dynamically setting `Access-Control-Allow-Origin` based on the incoming request.
