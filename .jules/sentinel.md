## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-02-03 - Insecure Default CSP Configuration
**Vulnerability:** The `Default` implementation for `SecurityHeadersConfig` included `'unsafe-inline'` and `'unsafe-eval'` directives, creating a security risk if `default()` was used without explicit environment-based overrides.
**Learning:** Default implementations for security-critical configurations should always fail safe (secure by default), requiring explicit opt-in for permissive settings (like development mode).
**Prevention:** Ensure `impl Default` provides the strictest possible configuration. Use factory methods (like `from_sources` or `new_dev`) for permissive variants.
