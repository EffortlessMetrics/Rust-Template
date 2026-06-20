## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-01-20 - Constant-Time String Comparison Vulnerability
**Vulnerability:** Found hand-rolled constant-time string comparison in `crates/http-auth-verifier/src/lib.rs` using `fold` and bitwise XOR (`acc | (x ^ y)`). Compiler optimizations can defeat these hand-rolled attempts and re-introduce timing attacks.
**Learning:** Never rely on hand-rolled iterators for constant-time cryptographic comparisons in Rust, as compiler optimizations can defeat constant-time guarantees.
**Prevention:** Always use vetted cryptography primitives like the `subtle` crate (`ConstantTimeEq`) for secure comparison operations.
