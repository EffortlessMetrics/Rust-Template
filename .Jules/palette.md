## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.
## 2026-04-26 - Native Search Inputs
**Learning:** The text input used for filtering the Acceptance Criteria Coverage table relied solely on a placeholder text. Converting `type="text"` to `type="search"` automatically provides native browser features (like a clear button) and better semantic meaning, while an explicit `aria-label` ensures screen readers can identify the field without external labels.
**Action:** Use `type="search"` with `aria-label` for all filter/search inputs instead of generic text fields.
