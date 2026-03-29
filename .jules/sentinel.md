## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-03-29 - Compiler Optimization Vulnerabilities in Constant-Time Comparisons
**Vulnerability:** The custom `constant_time_eq` implementation used a `fold` operation over bytes with `x ^ y`. While seemingly constant-time, modern compilers (like LLVM) can aggressively auto-vectorize or short-circuit such manual loops, potentially re-introducing timing side-channels.
**Learning:** Writing truly constant-time code in high-level languages requires explicit compiler directives (e.g., black boxes or inline assembly) to prevent optimizations.
**Prevention:** Never write manual constant-time comparison loops. Always use the `subtle` crate's `ConstantTimeEq` trait, which is specifically designed to guarantee constant-time evaluation against aggressive compiler optimizations.
