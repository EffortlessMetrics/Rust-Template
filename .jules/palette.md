## 2026-01-21 - Active Navigation States
**Learning:** Server-side rendered applications often neglect active navigation states because they require explicit logic to match routes, unlike client-side routers that handle it automatically. This degrades accessibility for screen reader users who rely on `aria-current="page"` to know where they are.
**Action:** Implement a reusable `nav_link` helper in SSR contexts that automatically applies `class="active"` and `aria-current="page"` based on the current page identifier.
