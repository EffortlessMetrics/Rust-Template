## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - Partial Domain Matching in CORS
**Vulnerability:** The CORS `is_origin_allowed` function matched wildcard subdomains using `origin.ends_with(wildcard_domain)`. This allowed `evilexample.com` to match `*.example.com` because it ends with `example.com`.
**Learning:** String suffix matching is insufficient for domain validation. A dot separator must be enforced to distinguish subdomains from partial domain matches.
**Prevention:** When implementing wildcard domain matching, always ensure the suffix match is preceded by a dot (e.g., `.example.com`) or matches the exact domain.
