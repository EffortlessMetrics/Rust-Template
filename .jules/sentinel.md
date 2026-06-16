## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-06-16 - DoS in Constant-Time Comparison Fix
**Vulnerability:** A constant-time comparison check used hand-rolled bitwise XOR which can be optimized away by the compiler or introduce subtle leaks.
**Learning:** Constant time comparisons should always be implemented using vetted cryptography primitives like the `subtle` crate to ensure constant-time execution is guaranteed by the compiler/LLVM.
**Prevention:** Use `subtle` crate for constant time checks.
