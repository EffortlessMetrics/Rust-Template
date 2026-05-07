## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-18 - [Production CSP Vulnerable to XSS]
**Vulnerability:** The default production Content-Security-Policy included `'unsafe-eval'` and `'unsafe-inline'` for `script-src` and `style-src`. This effectively nullifies the XSS protection CSP is supposed to provide.
**Learning:** CSP headers must be strict in production. While development often requires 'unsafe-eval' for HMR or dev tools, production must never include these. The codebase has separate configurations for development and production, but the default/production configuration was overly permissive.
**Prevention:** Ensure development-specific overrides are correctly scoped to the development environment, and that the default configuration is strict. Always avoid `unsafe-eval` and `unsafe-inline` in production.
