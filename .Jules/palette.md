## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-02-15 - Maud Escaping Inline Scripts/Styles
**Learning:** `maud` automatically escapes string content, which breaks inline JavaScript (e.g., `&&` becomes `&amp;&amp;`) and CSS (e.g., `>` becomes `&gt;`).
**Action:** Always wrap inline CSS, JavaScript, and pre-formatted content (like Mermaid diagrams) in `PreEscaped(...)` when using `maud` templates.
