## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-02-17 - Maud Script Escaping
**Learning:** Embedded CSS or JavaScript in `maud` templates via helper functions (e.g., `style { (styles()) }`) will be HTML-escaped by default, breaking code execution (e.g., `&&` becomes `&amp;&amp;`).
**Action:** Always wrap the return value of helper functions containing raw CSS/JS in `maud::PreEscaped()` (e.g., `style { (PreEscaped(styles())) }`).
