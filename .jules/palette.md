## 2026-06-22 - [Search Input Accessibility]
**Learning:** By default, text inputs acting as search boxes lack proper semantics and clear button functionality, reducing usability for screen reader users and general users.
**Action:** Always use `type="search"` instead of `type="text"` for search boxes, and ensure they have an explicit `aria-label` for better accessibility and native browser features.
