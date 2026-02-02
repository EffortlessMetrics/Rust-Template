## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-05-24 - Partial Domain Match in CORS
**Vulnerability:** A custom wildcard subdomain check `*.example.com` incorrectly allowed `evilexample.com` because it only checked `ends_with("example.com")` without ensuring the prefix ended with a dot.
**Learning:** Custom logic for wildcard domain matching is prone to subtle "partial match" bugs. The logic "ends with suffix" is insufficient for domain security.
**Prevention:** Always ensure that a suffix match is immediately preceded by a separator (like `.`) or matches the string exactly.
