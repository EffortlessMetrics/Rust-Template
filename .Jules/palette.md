## 2024-05-22 - Server-Side Rendering with Maud
**Learning:** This application uses the `maud` crate for server-side HTML rendering directly in Rust. This means UI components and styles are embedded in the Rust code (specifically `crates/app-http/src/platform/ui.rs`), not in separate template or CSS files. This approach offers type safety for HTML but requires recompilation for any UI change.
**Action:** When working on UX improvements, look for `html!` macros in Rust files. CSS is currently embedded in the `layout` function's `<style>` block.
