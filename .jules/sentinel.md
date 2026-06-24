## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-28 - XSS Risk in Production CSP Configuration
**Vulnerability:** The default production Content Security Policy (CSP) incorrectly included `'unsafe-eval'` and `'unsafe-inline'` directives, introducing a High-Severity Cross-Site Scripting (XSS) vulnerability.
**Learning:** Local development tools (like Vite or HMR) and frontend rendering may require permissive CSP directives to function properly. However, development-specific configurations must never leak into production defaults.
**Prevention:** Hardened the default production CSP to remove `'unsafe-eval'` and `'unsafe-inline'`. Used the `development()` constructor or environment variables via `from_sources()` instead to explicitly enable these directives for local environments, effectively isolating permissive configurations from production deployments.
