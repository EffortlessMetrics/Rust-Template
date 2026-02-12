## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-05-18 - CORS Wildcard Partial Match Vulnerability
**Vulnerability:** CORS configurations allowing `https://*.example.com` inadvertently allowed `https://evilexample.com` because the suffix match did not enforce a preceding dot separator.
**Learning:** `ends_with` checks for domain suffixes are insufficient for subdomain matching; the character boundary must be explicitly verified.
**Prevention:** When implementing wildcard subdomain matching, always verify that the matched suffix is preceded by a dot (`.`) or that the full string is an exact match.
