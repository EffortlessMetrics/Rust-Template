
## 2025-05-15 - ARIA live regions for dynamic filtering
**Learning:** The dynamic filter logic for the AC Coverage table was not announcing results to screen readers. It's crucial to include an `aria-live="polite"` element that explicitly announces the count of filtered items (e.g., "5 items found") when updates occur, rather than expecting screen readers to deduce the number of visible rows from DOM changes.
**Action:** Whenever implementing client-side filtering or search, always include an `aria-live` element (`.sr-only`) to announce the updated visible item count to screen readers.
