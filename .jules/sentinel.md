## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-02-13 - Partial Domain Matching in CORS Wildcards
**Vulnerability:** A CORS wildcard implementation allowed `*.example.com` to match `evilexample.com` because it only checked `origin.ends_with(wildcard_domain)`.
**Learning:** String suffix checks are insufficient for domain matching; domains are hierarchical and separated by dots. `*.example.com` implies `(anything).example.com`.
**Prevention:** When implementing wildcard domain matching, always ensure the matching suffix is immediately preceded by a dot (`.`) in the target string.
