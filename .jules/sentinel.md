## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-03-14 - Fix CORS Caching Vulnerability
**Vulnerability:** Missing `Vary: Origin` header in CORS middleware responses (both regular and preflight).
**Learning:** Without `Vary: Origin`, caching servers or CDNs might improperly cache a CORS response intended for a specific origin and mistakenly serve it to requests from other origins (or same-origin requests), leading to potential CORS cache poisoning or bypassed restrictions.
**Prevention:** Always append `Vary: Origin` (using `append` instead of `insert` to avoid overwriting existing `Vary` headers) to CORS responses when the origin is dynamically allowed or reflected.
