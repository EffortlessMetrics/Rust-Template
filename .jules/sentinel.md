## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-02-05 - Production CSP Vulnerability
**Vulnerability:** The default production Content Security Policy (CSP) allowed `'unsafe-inline'` and `'unsafe-eval'`.
**Learning:** Both development and production CSP configurations were identical in their fallback defaults. This permitted dangerous execution capabilities in production.
**Prevention:** Explicitly separate development configurations from production configurations for CSP settings, and rigorously test default fallback values for strictness.
