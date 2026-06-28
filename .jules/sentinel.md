## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2025-01-01 - [Constant-Time Comparison Vulnerability]
**Vulnerability:** A constant-time string comparison function (`constant_time_eq` in `http-auth-verifier`) used hand-rolled XOR operations with `fold`, which can be defeated by compiler optimizations, introducing a potential timing attack vulnerability.
**Learning:** Never use hand-rolled iterators for cryptographic constant-time comparisons. Compiler optimizations can short-circuit operations and negate constant-time guarantees. Always use vetted cryptography primitives like `subtle`.
**Prevention:** Use the `subtle` crate's `ConstantTimeEq` trait for all constant-time comparison operations.
