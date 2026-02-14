## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - CORS Wildcard Subdomain Vulnerability
**Vulnerability:** A CORS configuration using `*.example.com` allowed `evilexample.com` because the check only verified `origin.ends_with("example.com")` without ensuring a dot separator.
**Learning:** Naive suffix matching is insufficient for subdomain validation. Partial domain matches can lead to security bypasses.
**Prevention:** Always verify that the suffix match is preceded by a dot (`.`) or that the strings are identical.
