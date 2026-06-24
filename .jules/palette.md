
## 2025-02-13 - Avoid client-side DOMContentLoaded for initial UI states
**Learning:** In Maud templates used for `crates/http-platform/src/ui.rs` and `crates/app-http/src/platform/ui.rs`, relying on client-side JS (`DOMContentLoaded`) to set initial visual and accessibility states (like `active` class or `aria-pressed="true"`) causes a Flash of Unstyled Content (FOUC) and can mislead screen readers. Dynamic tables also require `aria-live="polite"` on their container to announce content updates asynchronously.
**Action:** Always render initial visual state and ARIA attributes server-side using Maud (e.g., `button.active aria-pressed="true"`). Use `aria-live="polite"` on containers where dynamic JS updates the DOM content.
