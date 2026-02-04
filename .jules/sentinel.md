## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - Partial Domain Match in CORS
**Vulnerability:** CORS middleware allowed `*.example.com` by checking `ends_with("example.com")`, which inadvertently allowed `evilexample.com`.
**Learning:** Suffix checks for domain names are insufficient for wildcard matching; the suffix must be preceded by a dot separator (or be an exact match) to prevent partial domain takeovers.
**Prevention:** When implementing wildcard domain checks, verify `origin == domain` OR `origin.ends_with("." + domain)`.
