## 2024-05-15 - Semantic Search Inputs
**Learning:** Search inputs should use `type="search"` rather than `type="text"` to provide native browser features (like clear buttons) and better semantic meaning. Additionally, they require an explicit `aria-label` (e.g., `aria-label="Search..."`) so screen readers can correctly identify them without external labels.
**Action:** When creating or reviewing search inputs, always verify they use `type="search"` and include an explicit `aria-label`.
