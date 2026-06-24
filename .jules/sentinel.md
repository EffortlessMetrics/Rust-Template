## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-04-30 - Remove Unsafe CSP Directives
**Vulnerability:** The production Content Security Policy (CSP) allowed `'unsafe-inline'` and `'unsafe-eval'` directives.
**Learning:** This could allow Cross-Site Scripting (XSS) attacks by allowing execution of inline scripts/styles and `eval()`.
**Prevention:** Avoid using `'unsafe-inline'` and `'unsafe-eval'` in production CSP headers. Use nonces or hashes for inline scripts and styles if strictly necessary, and avoid `eval()`.
