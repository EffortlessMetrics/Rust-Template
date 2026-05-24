## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-24 - Manual ARIA State Management
**Learning:** Client-side interactivity in `crates/http-platform/src/ui.rs` is implemented via embedded vanilla JavaScript. When CSS classes like `.active` are toggled, ARIA attributes (e.g., `aria-pressed`) must be manually updated in the same JS function to maintain accessibility.
**Action:** Always verify that JS handlers for UI state changes (filters, tabs) update corresponding ARIA attributes alongside visual class changes.
