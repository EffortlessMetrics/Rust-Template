## 2024-05-22 - Reusable Active Navigation State in Maud
**Learning:** In `maud` templates, handling active states for navigation links can be cleanly encapsulated in a local closure using `aria-current=[condition.then(|| "page")]`. This avoids repetitive `if/else` blocks and keeps the template declarative.
**Action:** Use local closures for repeated UI components that need context-aware attributes (like active state) instead of macros or separate functions if they are small and context-specific.
