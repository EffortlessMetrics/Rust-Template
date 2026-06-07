## 2024-06-07 - Improved Semantic Search Inputs
**Learning:** For search inputs, always use `type="search"` instead of `type="text"`. It provides native browser features (like clear buttons) and better semantic meaning. Ensure they include an explicit `aria-label` (e.g., `aria-label="Search..."`) so screen readers can correctly identify them without external labels.
**Action:** When adding or reviewing search inputs across the application, verify they use the correct `type` attribute and have appropriate accessibility labels.
