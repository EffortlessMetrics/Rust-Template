
## 2024-06-25 - Native Search Inputs for Accessibility
**Learning:** In the platform UI views (e.g., coverage), search inputs were using generic `type="text"` without explicit labels. This prevents screen readers from understanding their purpose without external labels, and misses out on native browser features like the clear ("x") button that come with `type="search"`.
**Action:** Always use `type="search"` for search fields and include an explicit `aria-label` (e.g., `aria-label="Search..."`) so screen readers correctly identify them, and users get native browser search features out-of-the-box.
