## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-01-31 - Filter Button Accessibility
**Learning:** For a set of mutually exclusive filter buttons, using `aria-pressed` (toggled via JS) is a valid and simple alternative to a `tablist` pattern, especially when buttons are visually separate.
**Action:** Use `aria-pressed` for toggleable filter controls.

## 2026-01-31 - Maud Inline Script Escaping
**Learning:** Embedding inline JavaScript or CSS in `maud` templates requires wrapping the content in `PreEscaped(...)` to prevent automatic HTML entity escaping (e.g., `&` becoming `&amp;` or `>` becoming `&gt;`).
**Action:** Always wrap inline scripts and styles in `PreEscaped(...)`.
