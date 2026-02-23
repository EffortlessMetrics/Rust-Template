## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-02-23 - Tool Integrity Verification Patterns
**Vulnerability:** Verifying checksums on *extracted binaries* against a registry of *tarball checksums* guarantees failure or requires insecure workarounds. Using `latest` tags in checksum files defeats the purpose of integrity verification.
**Learning:** Tool verification scripts must verify the downloaded archive (tarball/zip) *before* extraction to ensure supply chain integrity.
**Prevention:** Always checksum the exact artifact downloaded from the source. Never use mutable tags like `latest` in pinned tool registries.
