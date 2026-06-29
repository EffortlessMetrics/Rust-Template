
## 2026-06-29 - [Filter Button Accessibility]
**Learning:** Maud templates (`html!`) do not easily support dynamic boolean attributes evaluated conditionally at runtime via JavaScript.
**Action:** For UI components like filter buttons, establish static initial states (e.g., `aria-pressed="true"`) and update them using vanilla JS (e.g., `setAttribute('aria-pressed', 'true')`) to ensure proper accessibility communication to screen readers.
