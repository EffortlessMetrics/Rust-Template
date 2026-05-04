
## 2025-05-18 - Search Input Accessibility
**Learning:** Standard text inputs (`type="text"`) used as search boxes lack native browser features (like clear buttons) and fail to provide correct semantic meaning to screen readers. If there's no visible `<label>`, they are completely inaccessible.
**Action:** Always use `type="search"` instead of `type="text"` for search fields, and if there is no `<label>`, ensure an explicit `aria-label` is provided so screen readers can correctly identify the input.
