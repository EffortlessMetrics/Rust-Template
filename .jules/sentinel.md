# Sentinel Journal

## 2026-01-27 - Maud Template XSS and CSP Conflicts
**Vulnerability:** Found a stored DOM-based XSS vulnerability in `crates/app-http/src/platform/ui.rs` where user-controlled strings (AC scenarios) were injected into `innerHTML` without sanitization.
**Learning:** `maud` templates automatically escape content in `html!` blocks, but strings inside `script` blocks are also escaped unless wrapped in `PreEscaped`. This caused a double issue: unsafe manual `innerHTML` injection, and syntax errors when attempting to use sanitized strings in JS logic because `&` was escaped to `&amp;`.
**Prevention:** Always use `PreEscaped` for inline JavaScript in `maud` templates to prevent accidental syntax corruption, and implement manual escaping functions (like `escapeHtml`) when dealing with `innerHTML` injection within that JavaScript. Additionally, the default CSP configuration blocks external scripts (htmx/mermaid) which hinders functionality, though it is secure by default.
