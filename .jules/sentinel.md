## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-24 - DOM-based XSS in Platform UI
**Vulnerability:** The Platform UI used `row.innerHTML` to render AC coverage data (IDs, titles, etc.) fetched from an internal API. If spec data contained malicious scripts, this would execute them.
**Learning:** Even internal data sources should be treated as untrusted in the frontend. Using `innerHTML` for simple text insertion is a dangerous shortcut.
**Prevention:** Use `textContent` or `document.createElement()` for dynamic data insertion. Avoid `innerHTML` unless strictly necessary and combined with a sanitizer.
