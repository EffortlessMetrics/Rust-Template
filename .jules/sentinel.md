## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-08-07 - CORS Subdomain Partial Suffix Match
**Vulnerability:** The custom CORS implementation allowed partial suffix matches (e.g., `evilexample.com` matching `*.example.com`) because it only checked `origin.ends_with(wildcard_domain)` without verifying the boundary.
**Learning:** `ends_with` is insufficient for domain matching when using wildcards. Code duplication between `app-http` and `http-middleware` meant the vulnerability existed in two places.
**Prevention:** Always verify that a suffix match is preceded by a dot (`.`) or is an exact match. Avoid duplicating security-critical logic; centralize it in a shared library.
