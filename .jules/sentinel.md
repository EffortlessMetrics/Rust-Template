## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - Timing side channel in basic auth verification
**Vulnerability:** The `constant_time_eq` function in `crates/http-auth-verifier/src/lib.rs` used a manual `fold` with bitwise XOR (`left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0`).
**Learning:** Manual implementations of constant-time equality in Rust are notoriously prone to being optimized away or transformed by LLVM into non-constant-time operations (like short-circuiting or vectorization that leaks timing data), particularly in `--release` builds.
**Prevention:** Never use hand-rolled iterators or loops for cryptographic timing protection. Always use vetted cryptography primitives like the `subtle` crate (`ConstantTimeEq`), which uses inline assembly and compiler barriers to guarantee constant-time execution regardless of compiler optimizations.
