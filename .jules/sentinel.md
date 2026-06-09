## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.
## 2024-05-24 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times or checked all bytes explicitly via `.zip`, exposing a DoS vector where a large input would cause excessive CPU usage, or failing to safely protect equal-length comparisons due to optimizer interference.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities, or manual constant-time comparison algorithms can be defeated by modern compilers. Standard practice is to explicitly use community-vetted cryptography primitives like those in the `subtle` crate which provide reliable constant-time operations.
**Prevention:** Always use the `subtle` crate for constant-time comparisons (`ConstantTimeEq::ct_eq`) rather than hand-rolling logic like bitwise folds.
