## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-02-01 - Subdomain Wildcard Partial Match
**Vulnerability:** CORS origin validation used `ends_with` to check wildcard subdomains (e.g., `*.example.com`), allowing attackers to bypass protections using partial matches like `evilexample.com`.
**Learning:** Domain names are hierarchical with dot separators; string suffix matching ignores this structure and creates opportunities for "suffix takeover".
**Prevention:** When matching wildcard domains, explicitly verify that the matching suffix is immediately preceded by a dot (`.`) or is the exact domain itself.
