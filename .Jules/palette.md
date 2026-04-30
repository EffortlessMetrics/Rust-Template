## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.
## 2024-05-24 - Search Input Semantic Affordance
**Learning:** In Maud templates generating standard HTML inputs, using `type="text"` for search boxes loses native browser affordances (like the "X" clear button on WebKit browsers) and lacks proper semantic meaning. Search inputs also require an explicit `aria-label` to be announced properly by screen readers since they often lack a visible `<label>`.
**Action:** Always use `type="search"` for search inputs and ensure they include an explicit `aria-label` (e.g., `aria-label="Search..."`) matching the placeholder text to improve both visual and screen reader accessibility.
