## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-30 - Fix timing side-channel in basic token verification
**Vulnerability:** A basic timing attack side-channel was present in token verification. The codebase used a manually rolled `.fold()` operation for constant-time string comparison (`left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0`).
**Learning:** Manual implementations of constant-time functions in Rust are vulnerable to timing side-channels due to compiler optimizations like auto-vectorization or short-circuiting.
**Prevention:** Always use the `subtle` crate's `ConstantTimeEq` trait, which uses compiler black-boxes and inline assembly to guarantee constant-time evaluation.
