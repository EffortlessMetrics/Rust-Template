## 2024-04-05 - Missing ARIA labels on search inputs
**Learning:** Search inputs in the UI rely entirely on placeholders and lack associated `<label>` elements or `aria-label` attributes. This is an accessibility anti-pattern.
**Action:** Always ensure search inputs, particularly those generated in Maud templates (`ui.rs`), have an explicit `aria-label` added symmetrically across all UI rendering contexts.
