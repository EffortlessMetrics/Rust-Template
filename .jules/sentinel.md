## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - CORS Partial Wildcard Match
**Vulnerability:** The CORS middleware implementation allowed partial matches for wildcard subdomains (e.g., `https://*.example.com` matched `https://evilexample.com`) because it only checked `ends_with` without verifying the preceding character was a dot.
**Learning:** `ends_with` checks are insufficient for domain matching when wildcards are involved. A domain suffix match must always be anchored by a dot to prevent "suffix confusion" attacks.
**Prevention:** When implementing wildcard domain matching, explicitly check that the match is either exact or preceded by a dot (e.g., `origin.ends_with(suffix) && origin[len-suffix_len-1] == '.'`).
