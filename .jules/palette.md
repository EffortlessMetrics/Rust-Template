## $(date +%Y-%m-%d) - Use type="search" and aria-label for search inputs
**Learning:** Generic text inputs (`type="text"`) used for search often lack explicit `<label>` tags in this application's design system, making them inaccessible to screen readers.
**Action:** Always use `type="search"` instead of `type="text"` to provide native browser features (like clear buttons), and ensure an explicit `aria-label` is included when a visual `<label>` is absent.
