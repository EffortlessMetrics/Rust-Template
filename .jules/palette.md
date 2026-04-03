
## 2024-05-24 - Prevent FOUC and Screen Reader Misreporting with Server-Side UI State
**Learning:** In Maud templates, relying on client-side `DOMContentLoaded` scripts to set initial UI states (like `.active` classes or `aria-pressed="true"`) causes a Flash of Unstyled Content (FOUC) and can lead to screen readers temporarily misreporting the initial state before the JS executes.
**Action:** Always render initial UI states and attributes directly in the server-side Maud templates. Ensure dynamic JavaScript state updates toggle both visual classes and accessibility attributes (e.g., `aria-pressed`) simultaneously.
