## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-18 - [Fix Timing Attack in Token Verification]
**Vulnerability:** Hand-rolled constant-time string comparison using a bitwise fold iterator `left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0`.
**Learning:** Manual bitwise operations in Rust can be vectorized or optimized out by LLVM in ways that break constant-time guarantees, introducing timing attack vulnerabilities in credential verification.
**Prevention:** Always use vetted cryptography primitives like the `subtle` crate (`ConstantTimeEq`) for constant-time comparisons, which use compiler optimization barriers.
