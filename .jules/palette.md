## 2026-04-19 - Adding ARIA labels to search inputs
**Learning:** Relying solely on placeholders for search inputs is a common accessibility anti-pattern in the codebase's Maud templates. Screen readers need explicit labels.
**Action:** Always ensure `<input type="text">` elements acting as search boxes have an explicit `aria-label` attribute instead of relying entirely on the `placeholder` attribute.
