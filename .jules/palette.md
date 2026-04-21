
## 2026-04-21 - Accessibility of Search Inputs
**Learning:** Search input fields relying solely on `placeholder` text are considered an accessibility anti-pattern because screen readers often do not treat them as proper labels. They must include an explicit `aria-label` (e.g., `aria-label="Search coverage criteria"`) to be fully accessible.
**Action:** Ensure all search inputs or icon-only buttons include an `aria-label` during future UI or UX implementations.
