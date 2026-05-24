## 2024-05-23 - DoS in Constant-Time Comparison
**Vulnerability:** A `constant_time_eq` implementation iterated `max(a.len(), b.len())` times, exposing a DoS vector where a large input would cause excessive CPU usage.
**Learning:** Attempts to "avoid leaking length" by checking all bytes can inadvertently introduce algorithmic complexity vulnerabilities. Standard practice is to check length first (leaking length but preventing DoS) and then compare in constant time.
**Prevention:** Prefer `subtle` crate or idiomatic constant-time comparisons that explicitly handle length checks to bound execution time.

## 2024-05-22 - Manual DOM Sanitization Required
**Vulnerability:** DOM-based XSS in Platform UI
**Learning:** The project uses `maud` for server-side rendering but embeds raw vanilla JavaScript for client-side interactivity. This JS uses `innerHTML` to render API data without automatic sanitization.
**Prevention:** Always implement and use a local `escapeHtml` function when using `innerHTML` in embedded JS scripts. Prefer `textContent` where possible, but for mixed content (like badges inside cells), strict manual escaping is mandatory.
