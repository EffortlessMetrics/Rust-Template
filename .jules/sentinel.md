## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-18 - Remove unsafe-inline and unsafe-eval from default CSP
**Vulnerability:** The default Content Security Policy (CSP) used for the `SecurityHeadersConfig` included `'unsafe-inline'` and `'unsafe-eval'`.
**Learning:** `Default::default()` should be safe for production. The `production()` configuration was already safe, but any usage of `Default::default()` without explicit overrides would have insecure defaults.
**Prevention:** Always ensure the `Default` trait implementation for security configurations is strict and secure. Use separate methods for permissive development configurations.
