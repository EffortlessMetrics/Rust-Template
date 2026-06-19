
## 2024-05-01 - Search Input Accessibility
**Learning:** For search inputs, using `type="search"` instead of `type="text"` provides native browser features (like clear buttons) and better semantic meaning. Adding an explicit `aria-label` ensures screen readers can correctly identify them without external labels.
**Action:** Always use `type="search"` and include an explicit `aria-label` for search inputs.
