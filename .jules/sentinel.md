## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-18 - Constant-Time Equality Implementations
**Vulnerability:** A constant-time string comparison function `constant_time_eq` was implemented using a hand-rolled iterator pattern (`left.bytes().zip(right.bytes()).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0`).
**Learning:** Hand-rolled constant-time operations are risky because compiler optimizations can defeat the intended constant-time guarantees, potentially leading to timing attacks.
**Prevention:** Always use vetted cryptography primitives like the `subtle` crate (`ConstantTimeEq`) for constant-time comparisons, and explicitly check the length first to prevent algorithmic complexity DoS vulnerabilities.
