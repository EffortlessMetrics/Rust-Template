## 2026-01-24 - DOM XSS in maud-rendered pages
**Vulnerability:** Client-side JavaScript in `maud` templates used `innerHTML` to render data fetched from an API without sanitization.
**Learning:** `maud` escapes string literals inside `script` tags unless wrapped in `PreEscaped`. This ensures safety by default but breaks valid JavaScript (e.g., `=>` becomes `=&gt;`).
**Prevention:** Use `PreEscaped` for JavaScript blocks in `maud` templates, but ensure any user data injected into DOM within that JS is explicitly sanitized (e.g. using a custom `escapeHtml` function).
