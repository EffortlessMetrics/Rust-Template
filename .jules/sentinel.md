## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-04-18 - Timing Attack via Compiler Loop Optimization
**Vulnerability:** A custom `constant_time_eq` implementation using `left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0` is susceptible to timing attacks.
**Learning:** The Rust compiler (LLVM) might optimize the loop via vectorization or short-circuiting, violating the constant-time guarantee and enabling timing attacks on token equality comparison.
**Prevention:** Always use established cryptographic libraries like the `subtle` crate (`subtle::ConstantTimeEq`) instead of rolling custom constant-time checks.
