## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Overly Permissive CSP
**Vulnerability:** The production Content Security Policy (CSP) included `'unsafe-inline'` and `'unsafe-eval'` directives, significantly increasing the risk of Cross-Site Scripting (XSS) by allowing inline scripts and `eval()` execution.
**Learning:** Default configurations for security headers must be split between development (where `unsafe-inline` might be needed for HMR/tooling) and production (which must be strictly locked down).
**Prevention:** Always verify that production security headers enforce the principle of least privilege, specifically ensuring CSP directives like `script-src` and `style-src` do not use `'unsafe-*'` flags in production environments.
