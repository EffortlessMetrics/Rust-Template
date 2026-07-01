## 2024-05-18 - Maud template conditional attributes
**Learning:** Maud templates (`html!`) do not easily support dynamic boolean attributes evaluated conditionally at runtime via JavaScript. For UI components like filter buttons, establish static initial states (e.g., `aria-pressed="true"`) and update them using vanilla JS (e.g., `setAttribute('aria-pressed', 'true')`) to ensure proper accessibility communication.
**Action:** Always set static initial attributes in Maud templates and use vanilla JS for dynamic updates.
