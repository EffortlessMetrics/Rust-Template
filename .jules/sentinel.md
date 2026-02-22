## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - CORS Wildcard Subdomain Vulnerability
**Vulnerability:** The CORS configuration allowed partial domain matches for wildcard subdomains (e.g., `*.example.com` allowed `evilexample.com`) due to simple suffix matching without verifying the separator.
**Learning:** When implementing wildcard matching, always ensure that the wildcard suffix is preceded by the domain separator (e.g., `.`) to prevent "super-domain" attacks.
**Prevention:** Explicitly check for the dot separator when validating subdomain wildcards, or use a robust URL matching library instead of simple string suffix checks.
