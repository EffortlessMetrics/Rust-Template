## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-23 - Timing Attacks in Constant-Time Comparison
**Vulnerability:** A custom implementation of `constant_time_eq` using `left.bytes().zip(right.bytes()).fold(...)` was vulnerable to timing attacks due to compiler optimizations (like auto-vectorization or short-circuiting).
**Learning:** Manual implementations of constant-time functions in Rust are often optimized by LLVM in ways that introduce timing side-channels.
**Prevention:** Always use the `subtle` crate's `ConstantTimeEq` trait, which guarantees constant-time evaluation using compiler black-boxes and inline assembly.
