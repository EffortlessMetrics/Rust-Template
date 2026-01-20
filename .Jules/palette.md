## 2026-01-20 - Active Navigation State
**Learning:** `maud` templates in Rust require specific conditional logic (`@if`) for optional attributes like `aria-current`. Unlike JSX where you might use `{isActive ? "page" : undefined}`, `maud` prefers separate branches or helper functions to maintain type safety and clean templates.
**Action:** Use helper functions for repetitive UI elements that require conditional attributes to keep the main layout clean.
