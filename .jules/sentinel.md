## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-23 - Side-channel Timing Attacks in Constant-Time Code
**Vulnerability:** A manual implementation of `constant_time_eq` using `fold` over bytes is vulnerable to compiler optimizations (like auto-vectorization or short-circuiting) that can introduce side-channel timing attacks, leaking secrets.
**Learning:** Manual implementations of constant-time comparisons in Rust are unsafe. The compiler can optimize out seemingly "constant-time" loops.
**Prevention:** Always use the `subtle` crate's `ConstantTimeEq` trait, which uses compiler black-boxes and inline assembly to guarantee constant-time evaluation against compiler optimizations.
