## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-28 - Missing Accessible Labels
**Learning:** Found inputs without accessible labels on the platform UI pages. Search inputs with only `placeholder` attribute are not fully accessible to screen readers. In `maud` templates, `aria-label` attribute can be safely added to input elements without breaking formatting or structure.
**Action:** Always add explicit `aria-label` attributes to inputs that lack a visible `<label>` element.
