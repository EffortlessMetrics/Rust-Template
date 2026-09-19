## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-02-14 - Fix Default CSP Header
**Vulnerability:** The default Content-Security-Policy (CSP) headers included `unsafe-inline` and `unsafe-eval` for scripts and `unsafe-inline` for styles. This could allow Cross-Site Scripting (XSS) attacks in production if an attacker can inject malicious scripts or styles, bypassing the main protections of CSP.
**Learning:** The default configuration struct was explicitly defining these unsafe directives. While useful for local development or legacy setups without explicit configuration, default security policies should be restrictive ('secure by default') rather than permissive.
**Prevention:** Always follow 'secure by default' principles when defining default values for security-related structures. Use environment-specific overrides (e.g., `development()` methods) for permissive configurations rather than degrading the base defaults.
