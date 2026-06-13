## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - Removing Unsafe Directives from CSP
**Vulnerability:** The default Content Security Policy (CSP) allowed `'unsafe-inline'` and `'unsafe-eval'` in the `script-src` directive, leaving the application vulnerable to Cross-Site Scripting (XSS) if other vulnerabilities were exploited.
**Learning:** Default configurations often prioritize developer convenience over strict security. When auditing security headers, ensure that production environments strictly forbid `unsafe-*` directives.
**Prevention:** Always verify that security policies like CSP are as restrictive as possible in production (`production()` or `default()`), while explicitly relaxing them only in development (`development()`).
