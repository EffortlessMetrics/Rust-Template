# Palette's Journal

## 2026-01-24 - Accessible Navigation Active States in SSR
**Learning:** For server-side rendered applications using `maud`, managing active navigation states requires calculating the current page context at the layout level. Explicitly setting `aria-current="page"` alongside visual styling is critical for screen reader users who cannot perceive visual cues like underlines or color changes.
**Action:** Always implement a `nav_link` helper in SSR templates that accepts the current page context and automatically applies both the visual `.active` class and the semantic `aria-current="page"` attribute.
