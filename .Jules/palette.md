## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-04-29 - Improve Search Accessibility and UX
**Learning:** Using `type="text"` for search inputs misses out on native browser features like the clear button and correct semantic meaning.
**Action:** Always use `type="search"` instead of `type="text"` for search inputs and ensure they have an explicit `aria-label` (e.g., `aria-label="Search..."`) so screen readers can correctly identify them without external labels.
