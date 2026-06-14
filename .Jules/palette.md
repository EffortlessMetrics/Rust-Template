## 2025-04-25 - Use semantic search inputs
**Learning:** Using `type="search"` instead of `type="text"` provides built-in browser features (like clear buttons on mobile) and better semantic meaning, while an explicit `aria-label` ensures screen readers can identify standalone search fields without external labels.
**Action:** Always prefer `type="search"` over `type="text"` for filtering/search boxes and include an explicit `aria-label` or referenced `<label>`.
