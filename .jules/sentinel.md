## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - Default Production CSP Allows XSS
**Vulnerability:** The default Content Security Policy (CSP) configurations in the `http-middleware` and `app-http` crates permitted `'unsafe-inline'` and `'unsafe-eval'` directives, creating a high-severity Cross-Site Scripting (XSS) risk in production environments.
**Learning:** Security defaults must be uniformly strict by default. Permissive configurations (such as those needed for local development tools) should never be the `Default` implementation, but explicitly requested via designated constructors (e.g., `development()`).
**Prevention:** Harden `Default` trait implementations for security primitives. Validate that relaxed security settings are explicitly conditioned on a local or development environment context.
