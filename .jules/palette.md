## 2024-04-24 - Accessibility improvements for coverage page search

**Learning:** Search inputs in `app-http` and `http-platform` templates (using Maud) lacking explicit labels or ARIA labels fail accessibility standards, even if a `placeholder` exists. The codebase has duplicated UI implementations requiring symmetric updates. Playwright UI tests must use `#search-box` explicitly rather than relying on role "searchbox" when `type="text"` is present.

**Action:** Ensure both UI modules (`crates/app-http/src/platform/ui.rs` and `crates/http-platform/src/ui.rs`) are checked when applying Maud template fixes. Add `aria-label` directly to input fields when a visual `<label>` is not present in the design.
