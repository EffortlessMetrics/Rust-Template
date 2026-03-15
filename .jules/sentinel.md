## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2026-03-15 - CORS Cache Poisoning via Missing Vary Header
**Vulnerability:** The CORS middleware reflected the incoming `Origin` header into the `Access-Control-Allow-Origin` response header but failed to include `Vary: Origin`. This could lead to an intermediate cache serving a response intended for one origin to a completely different origin.
**Learning:** Whenever an HTTP response header's value is generated based on a request header (like `Origin`), the response MUST include a `Vary` header naming that request header.
**Prevention:** Explicitly `append` `Vary: Origin` to the response headers before inserting `Access-Control-Allow-Origin`. We must use `append` rather than `insert` for `Vary` because multiple middleware components might need to add their own `Vary` values.
