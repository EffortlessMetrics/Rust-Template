## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-23 - Initial UI States and Accessibility
**Learning:** Setting initial UI states (like `.active` classes and `aria-pressed` attributes) via client-side JavaScript on `DOMContentLoaded` causes a Flash of Unstyled Content (FOUC) and can lead to screen readers misreporting the initial state before the JS executes.
**Action:** Always render initial UI states server-side in Maud templates to ensure immediate accessibility and prevent FOUC. Dynamically update these states with JS only in response to user interactions.
