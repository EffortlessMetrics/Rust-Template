## 2026-01-23 - Active Navigation in Server-Side Rust
**Learning:** `maud` templates (v0.27.0) handle conditional attributes elegantly with `aria-current=[active.then(|| "page")]`, but standard `if` blocks cannot be used directly inside attribute definitions.
**Action:** Use `then()` or `then_some()` with Option return types for conditional HTML attributes in Rust/Maud UI code.
