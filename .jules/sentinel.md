## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2025-05-27 - CORS Wildcard Subdomain Bypass
**Vulnerability:** The CORS configuration `https://*.example.com` matched `https://evilexample.com` because the implementation checked `origin.ends_with(wildcard_domain)` where `wildcard_domain` was `example.com` (extracted without the preceding dot).
**Learning:** When implementing wildcard matching logic manually (instead of using established libraries), off-by-one errors in string slicing can lead to critical security gaps. The assumption that `*.` always implies a subdomain boundary must be explicitly enforced in the code.
**Prevention:** Explicitly include the separator (dot) in the suffix check or verify that the character preceding the suffix match is a dot.
