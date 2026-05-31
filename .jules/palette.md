## 2024-11-20 - Update search input to use type="search" and add aria-label
**Learning:** Using `type="text"` for search inputs lacks browser-native search features (like clear buttons) and semantic meaning, particularly when an explicit `<label>` is missing. Placeholder text alone is insufficient for screen readers.
**Action:** Always use `type="search"` with an explicit `aria-label` (e.g., `aria-label="Search..."`) for search inputs to ensure accessibility and better user experience without requiring custom UI components.
