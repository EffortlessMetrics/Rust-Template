## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2023-11-20 - Server-Side State Rendering for Screen Readers
**Learning:** Initial UI states (like `.active` classes and `aria-pressed="true"`) shouldn't rely on `DOMContentLoaded` client-side JS. The screen reader parses the DOM immediately and can misreport the initial state of the UI components (like a toggle filter button) if it's rendered neutrally on the server and then toggled via JS on load.
**Action:** When working in `maud` templates for this app, ensure initial state logic (classes, ARIA attributes) is rendered fully on the server to prevent FOUC and guarantee immediate accessibility accuracy.
