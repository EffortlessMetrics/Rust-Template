## 2026-04-11 - Replaced text input with search input
**Learning:** In this application's UI, using `type="text"` combined with a `placeholder` for search boxes is an accessibility anti-pattern. Search boxes must use `type="search"` to provide native browser affordances (like the clear button) and must have an explicit `aria-label` for screen readers since placeholders are not reliable labels.
**Action:** When implementing or reviewing search inputs in Maud templates, always ensure they use `type="search"` and include an `aria-label` attribute.
