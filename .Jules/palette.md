## 2024-05-22 - Maud Conditional Attributes
**Learning:** `maud` templates in this codebase require the `attr=[Option]` syntax for conditional attributes like `aria-current`. Standard `if` blocks cannot be used inside attribute lists.
**Action:** Use `attr=[condition.then(|| "value")]` for all conditional ARIA attributes.
## 2026-04-28 - UI Route Endpoints for Verification
**Learning:** When writing Playwright UI verification scripts for the local dev server, note that `/platform/...` routes (e.g., `/platform/coverage`) return JSON API responses, whereas `/ui/...` routes (e.g., `/ui/coverage`) serve the actual HTML pages.
**Action:** Ensure navigation targets the correct HTML endpoint (`/ui/...`) instead of the API endpoint when verifying visual changes.
