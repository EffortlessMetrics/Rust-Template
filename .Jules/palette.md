## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-23 - Initial ARIA states
**Learning:** Initializing ARIA attributes like `aria-pressed` or `aria-current` via client-side JavaScript (`DOMContentLoaded`) causes a Flash of Unstyled Content (FOUC) and can lead screen readers to announce incorrect initial states before the script executes.
**Action:** Always render initial ARIA states directly on the server (e.g., using `maud` server-side rendering) to ensure immediate accessibility and correct visual state upon page load.
