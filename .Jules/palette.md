## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-23 - Interactive Filter Accessibility
**Learning:** Client-side filtering in `maud` templates uses vanilla JS to toggle classes. These interactions often miss ARIA state updates.
**Action:** When implementing toggle buttons (like filters), always pair class toggles with `aria-pressed` updates in the corresponding JS function.
