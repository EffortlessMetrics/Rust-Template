## 2024-05-18 - Improved Search Input Accessibility
**Learning:** Search inputs in this application's coverage views lacked native semantic `type="search"` and aria-labels, making them less accessible to screen readers and lacking native clear buttons.
**Action:** Always use `type="search"` instead of `type="text"` for search boxes, and provide explicit `aria-label` attributes to ensure screen readers announce the input's purpose correctly without needing an external visible label.
