## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.

## 2024-05-24 - Vanilla JS State Management
**Learning:** For server-rendered pages using vanilla JS for interactivity (like filters), visual class toggling is insufficient. Screen readers need state attributes updated programmatically.
**Action:** In the JS event handler, always pair `element.classList.toggle()` with `element.setAttribute('aria-pressed', isPressed)` to keep the semantic state in sync.
