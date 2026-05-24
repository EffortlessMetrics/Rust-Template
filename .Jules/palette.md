## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-02-11 - Maud Escaping for Scripts and Styles
**Learning:** `maud` templates escape strings by default, which breaks inline CSS and JS (e.g., `>` becomes `&gt;`). To render raw content, wrap strings in `PreEscaped(...)`.
**Action:** Always use `PreEscaped(...)` for `style` and `script` content in `maud` templates.
