
## 2024-03-20 - Use `type="search"` and explicit `aria-label` for search boxes
**Learning:** The native search boxes on the local dev server used `type="text"`, which misses out on native browser features like the clear button and better semantic meaning. Also, without an explicit `aria-label`, screen readers have trouble identifying the search boxes accurately.
**Action:** Always use `type="search"` instead of `type="text"` for search inputs and ensure they include an explicit `aria-label` (e.g., `aria-label="Search..."`).
