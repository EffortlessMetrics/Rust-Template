## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-04-03 - [Fix CORS Cache Poisoning Risk]
**Vulnerability:** Dynamically reflecting the `Origin` header into the `Access-Control-Allow-Origin` response header without appending `Vary: Origin`.
**Learning:** If a server reflects `Origin` for CORS without `Vary: Origin`, intermediate caches (like CDNs or proxies) may cache a response intended for `https://attacker.com` and mistakenly serve it to `https://victim.com`, leading to CORS cache poisoning and potential cross-origin data leakage.
**Prevention:** Whenever generating a dynamic `Access-Control-Allow-Origin` header based on the incoming `Origin`, always simultaneously append the `Vary: Origin` header.
