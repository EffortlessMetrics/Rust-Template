## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2026-02-23 - Filter State Accessibility
**Learning:** Filter buttons that act as mutually exclusive options are more intuitive when styled and announced as toggles (`aria-pressed`) rather than simple buttons, especially when they visually update the content without a full page reload.
**Action:** Use `aria-pressed="true/false"` for client-side filter controls and ensure visual state matches the ARIA state.

## 2026-02-23 - Maud Raw Content Escaping
**Learning:** `maud` templates automatically escape dynamic string content inside `<script>` and `<style>` tags, breaking JS syntax (like `=>`) and CSS selectors (like `[attr="val"]`).
**Action:** Always wrap dynamic CSS/JS content in `PreEscaped(...)` when injecting into `style` or `script` blocks in `maud` templates.
