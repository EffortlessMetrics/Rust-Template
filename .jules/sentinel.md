## 2025-02-14 - Fix Time-Based Side-Channel Vulnerability in Token Comparison
**Vulnerability:** The codebase compared basic authentication tokens using `fold` and `|` with bitwise XOR to compare strings character-by-character. While this attempts to be constant-time, standard Rust iterators and compiler optimizations can still short-circuit or introduce timing variances that could leak token bytes.
**Learning:** Hand-rolled constant-time string comparisons in authentication logic are unsafe due to compiler optimizations and lack of strict constant-time guarantees.
**Prevention:** Always use vetted cryptography primitives like the `subtle` crate's `ConstantTimeEq` for comparing secrets, tokens, and hashes to prevent timing attacks.
