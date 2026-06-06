## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-06-06 - Production Content Security Policy XSS Risk
**Vulnerability:** The default (production) Content Security Policy (CSP) in `crates/http-middleware/src/security_headers.rs` and `crates/app-http/src/middleware/security_headers.rs` included `'unsafe-inline'` and `'unsafe-eval'` directives for scripts and styles.
**Learning:** These directives disable key protections of CSP, making the application vulnerable to Cross-Site Scripting (XSS) attacks. They were likely copy-pasted from or conflated with the development configuration, which legitimately requires them for local development tools (e.g., Vite, HMR).
**Prevention:** Maintain strict separation between production and development security configurations. Production CSPs must never include `'unsafe-inline'` or `'unsafe-eval'`. Use `SecurityHeadersConfig::development()` or explicit feature flags for development environments instead of weakening the default production configuration.
