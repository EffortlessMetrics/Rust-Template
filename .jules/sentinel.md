## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-06-27 - XSS in Content Security Policy
**Vulnerability:** The default production Content Security Policy (CSP) included 'unsafe-inline' and 'unsafe-eval' in script-src and style-src directives.
**Learning:** Development CSP configurations frequently bleed into production defaults if strictness is not explicitly enforced. Permitting 'unsafe-inline' and 'unsafe-eval' severely undermines CSP protections against Cross-Site Scripting (XSS).
**Prevention:** Ensure production CSP strictly defines directives without 'unsafe-*' keywords. Apply permissive CSP solely in explicitly configured development environments.
