## 2024-05-09 - Search Input Semantics and Accessibility
**Learning:** Changing a generic search box from `<input type="text">` to `<input type="search">` enables native browser features like clear buttons ('x'), and adding an explicit `aria-label` ensures screen readers can identify the input correctly when it lacks an external label, providing a quick but impactful micro-UX win.
**Action:** When auditing forms, specifically look for text inputs used for searching and upgrade them to `type="search"` with appropriate ARIA labels, especially if they only rely on placeholder text.
