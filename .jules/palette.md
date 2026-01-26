## 2026-01-26 - Maud Component Pattern
**Learning:** In `maud` server-side rendering, defining closures (e.g., `let nav_link = |...|`) inside the handler/layout function is an effective way to create reusable, state-aware UI components without externalizing them to separate functions.
**Action:** Use local closures for small, context-dependent UI helpers (like navigation links checking current page ID) to keep code collocated and readable.
