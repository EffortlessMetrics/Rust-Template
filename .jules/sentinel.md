## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-06-21 - Compiler optimization of hand-rolled constant-time checks
**Vulnerability:** Hand-rolled constant-time string comparisons (e.g., using `fold` and XOR) can be optimized by the compiler into variable-time operations, defeating their purpose and introducing timing attacks.
**Learning:** The Rust compiler is smart enough to optimize manual bitwise operations in ways that break constant-time guarantees.
**Prevention:** Never use hand-rolled iterators for constant-time comparisons. Always use vetted cryptography primitives like the `subtle` crate's `ConstantTimeEq` trait.
