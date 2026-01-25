# Palette's Journal

## 2026-01-25 - Active Navigation State in Maud
**Learning:** `maud` templates allow defining closures (like `nav_link`) within the Rust code block to create reusable UI components with logic (e.g., conditional `active` class and `aria-current`). This is cleaner than repeating conditional logic for every link.
**Action:** Use local closures for repetitive UI elements that require state-dependent attributes (like navigation or tabs) to ensure consistency in accessibility attributes (`aria-current`).
