## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-24 - Server-Side State Rendering
**Learning:** Rendering initial UI state (like active classes or `aria-pressed`) in the server-side template avoids Flash of Unstyled Content (FOUC) and simplifies client-side JS by removing initialization logic.
**Action:** always bake the initial state into the HTML (e.g. in `maud`) rather than setting it via `DOMContentLoaded`.
